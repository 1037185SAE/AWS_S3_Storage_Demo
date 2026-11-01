#![allow(static_mut_refs)]

mod dialog;

use std::process::{Command, Output};
use std::io::{self, BufRead, Write};

const WINDOW_DIMENSIONS: egui::Vec2 = egui::vec2(500., 510.);

static mut ACCESS_KEY: String = String::new();
static mut SECRET_ACCESS_KEY: String = String::new();
static mut DEFAULT_REGION: String = String::new();
static mut ALLOC_FOLDER: String = String::new();
static mut BUCKET_NAME: String = String::new();
static mut ACCOUNT_ID: String = String::new();

macro_rules! access_static {
    ($val:tt > $static:tt) => {
        unsafe { $static = $val.clone(); }
    };
    ($val:tt < $static:tt) => {
        unsafe { $val = $static.clone(); }
    }
}

#[derive(Default)]
struct AppMain {
    objects: String,
    download_query: String,
    file_download_query: String,
    upload_query: String
}

impl eframe::App for AppMain {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        fn subtitle(text: &str) -> egui::RichText {
            egui::RichText::new(text).size(12.)
        }
        
        let alloc_folder;
        let bucket_name;
        let account_id;
        let access_key;
        let secret_access_key;
        let default_region;

        access_static!(alloc_folder < ALLOC_FOLDER);
        access_static!(bucket_name < BUCKET_NAME);
        access_static!(account_id < ACCOUNT_ID);
        access_static!(access_key < ACCESS_KEY);
        access_static!(secret_access_key < SECRET_ACCESS_KEY);
        access_static!(default_region < DEFAULT_REGION);
        
        if self.download_query.is_empty() {
            self.download_query = "<NONE>".to_string();
        }
        
        if self.upload_query.is_empty() {
            self.upload_query = "<NONE>".to_string();
        }

        if self.file_download_query.is_empty() {
            self.file_download_query = "<NONE>".to_string();
        }
        
        let title = egui::RichText::new("Simple S3 Demo").size(12.);
        ui.label(title);
        ui.separator();

        ui.label("List Objects");
        egui::ScrollArea::both()
            .auto_shrink(false)
            .max_height(120.)
            .show(ui, |ui| {
                let text_box = egui::TextEdit::multiline(&mut self.objects)
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .desired_rows(16)
                    .font(egui::FontId::monospace(16.));

                ui.add(text_box);
            }
        );

        if ui.button("List Objects").clicked() {
            let cmd = format!("aws s3api list-objects-v2 --bucket {} --prefix {} --query Contents[].Key --output text", bucket_name, alloc_folder);
            println!("Running command: {}", cmd);

            let output = run_command_with_envs(cmd.as_str(), vec![
                ("AWS_ACCESS_KEY_ID", access_key.as_str()),
                ("AWS_SECRET_ACCESS_KEY", secret_access_key.as_str()),
                ("AWS_ACCOUNT_ID", account_id.as_str()),
                ("AWS_DEFAULT_REGION", default_region.as_str())
            ]).unwrap();
            if ! output.status.success() {
                dialog::dialog_error("List Failed", "Failed to list objects. Check console for details");
                eprintln!("aws exited with error code: {}", output.status.code().unwrap());
            } else {
                dialog::dialog_info("Success", "Objects list retrieval successful");
                self.objects = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
                self.objects = self.objects.split_whitespace().filter(|f| f.len() > 0).collect::<Vec<&str>>().join("\n");
            }
        }

        ui.separator();
        ui.label("Upload Objects");

        let selection_result = subtitle("Selected File");
        ui.label(selection_result);

        let text_box = egui::TextEdit::singleline(&mut self.upload_query)
            .interactive(false)
            .desired_width(f32::INFINITY)
            .font(egui::FontId::monospace(16.));
        
        ui.add(text_box);

        if ui.button("Select item to upload").clicked() {
            let path = rfd::FileDialog::new().set_title("Select a file to upload").pick_file();
            
            if let Some(path) = path {
                self.upload_query = path.into_string().unwrap();
            }
        }

        if ui.button("Upload").clicked() {
            if self.upload_query == "<NONE>" {
                dialog::dialog_error("Upload failed", "Please select a file to upload");
            } else {
                let cmd = format!("aws s3 cp {} s3://{}/{}/", self.upload_query, bucket_name, alloc_folder);
                println!("Running command: {}", cmd);

                let output = run_command_with_envs(cmd.as_str(), vec![
                    ("AWS_ACCESS_KEY_ID", access_key.as_str()),
                    ("AWS_SECRET_ACCESS_KEY", secret_access_key.as_str()),
                    ("AWS_ACCOUNT_ID", account_id.as_str()),
                    ("AWS_DEFAULT_REGION", default_region.as_str())
                ]).unwrap();
                if ! output.status.success() {
                    dialog::dialog_error("Upload Failed", "Failed to upload object. Check console for details");
                    eprintln!("aws exited with error code: {}", output.status.code().unwrap());
                } else {
                    dialog::dialog_info("Success", "Object upload successful");
                }
            }
        }

        if ui.button("Clear Upload Selection").clicked() {
            self.upload_query.clear();
        }

        ui.separator();
        ui.label("Download Objects");

        let selection_result = subtitle("Selected File");
        ui.label(selection_result);

        let text_box = egui::TextEdit::singleline(&mut self.download_query)
            .interactive(false)
            .desired_width(f32::INFINITY)
            .font(egui::FontId::monospace(16.));
        
        ui.add(text_box);

        if ui.button("Select folder to download to").clicked() {
            let path = rfd::FileDialog::new().set_title("Select a folder to download to").pick_folder();
            
            if let Some(path) = path {
                self.download_query = path.into_string().unwrap();
            }
        }
        
        let selection_result = subtitle("Selected S3 Object");
        ui.label(selection_result);

        let text_box = egui::TextEdit::singleline(&mut self.file_download_query)
            .interactive(true)
            .desired_width(f32::INFINITY)
            .font(egui::FontId::monospace(16.));

        ui.add(text_box);

        if ui.button("Download").clicked() {
            if self.download_query == "<NONE>" || self.file_download_query == "<NONE>" {
                dialog::dialog_error("Upload failed", "Please select a folder to download to, and a file to download");
            } else {
                let cmd = format!("aws s3 cp s3://{}/{}/{} {}", bucket_name, alloc_folder, self.file_download_query, self.download_query);
                println!("Running command: {}", cmd);

                let output = run_command_with_envs(cmd.as_str(), vec![
                    ("AWS_ACCESS_KEY_ID", access_key.as_str()),
                    ("AWS_SECRET_ACCESS_KEY", secret_access_key.as_str()),
                    ("AWS_ACCOUNT_ID", account_id.as_str()),
                    ("AWS_DEFAULT_REGION", default_region.as_str())
                ]).unwrap();

                if ! output.status.success() {
                    dialog::dialog_error("List Failed", "Failed to download object. Check console for details");
                    eprintln!("aws exited with error code: {}", output.status.code().unwrap());
                } else {
                    dialog::dialog_info("Success", "Object download successful");
                }
            }
        }

        if ui.button("Clear Download Selection").clicked() {
            self.download_query.clear();
            self.file_download_query.clear();
        }
    }
}

#[inline(always)]
fn run_command_with_envs(cmd: &str, envs: Vec<(&str, &str)>) -> Result<Output, io::Error> {
    let argv = cmd.split(' ').collect::<Vec<&str>>();
    let prog = argv[0];
    let prog_args = &argv[1..];
    let mut cmd = Command::new(prog);
    cmd.args(prog_args);
    cmd.stdout(std::process::Stdio::piped());

    for (key, val) in envs {
        cmd.env(key, val);
    }

    Ok(cmd.spawn()?.wait_with_output()?)
}

fn prompt_user(prompt: &str) -> Result<String, std::io::Error> {
    print!("{}", prompt);
    std::io::stdout().lock().flush()?;

    let mut input_buf = String::new();
    std::io::stdin().lock().read_line(&mut input_buf)?;
    input_buf = input_buf.trim().to_string();
    Ok(input_buf)
}

fn main() -> Result<(), io::Error> {
    let alloc_folder = prompt_user("Allocated folder: ")?;
    let bucket_name = prompt_user("Bucket name: ")?;
    let account_id = prompt_user("Account ID: ")?;
    let access_key = prompt_user("Access Key: ")?;
    let secret_access_key = rpassword::prompt_password("Secret Access Key (keypresses are not echoed): ")?;

    let default_region = {
        let region = prompt_user("Default region (input nothing for default: us-east-1): ")?;

        if region == "" {
            "us-east-1".to_string()
        } else {
            region
        }
    };

    access_static!(alloc_folder > ALLOC_FOLDER);
    access_static!(bucket_name > BUCKET_NAME);
    access_static!(account_id > ACCOUNT_ID);
    access_static!(access_key > ACCESS_KEY);
    access_static!(secret_access_key > SECRET_ACCESS_KEY);
    access_static!(default_region > DEFAULT_REGION);

    let viewport = egui::viewport::ViewportBuilder::default()
        .with_resizable(false)
        .with_active(true)
        .with_inner_size(WINDOW_DIMENSIONS);

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = viewport;

    eframe::run_native(
        "Simple S3 Storage Demo",
        native_options,
        Box::new(|_cc| Ok(Box::new(AppMain::default())))
    ).unwrap();

    Ok(())
}