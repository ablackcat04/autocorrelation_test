use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use egui::{Color32, Pos2, Rect, Vec2};
use std::{sync::{Arc, Mutex}, time::Duration};


struct MyApp {
    values: Arc<Mutex<Vec<f32>>>
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            values: Arc::new(Mutex::new(vec![0f32; 7]))
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            let mut temp_values = self.values.lock().unwrap().clone();

            let mut vmax = 0f32;
            for i in &temp_values {
                if vmax < i.abs() {
                    vmax = i.abs();
                }
            }

            if vmax < 1f32 {
                vmax = 1f32;
            }

            let mut tv:Vec<f32> = Vec::new();
            for i in &temp_values {
                tv.push(i.abs() / vmax);
            }

            // ui.label(format!("{:?}", temp_values));

            let available_space = ui.available_size();

            let (_id, rect) = ui.allocate_space(available_space);

            let painter = ui.painter_at(rect);

            

            // Calculate the top-left and bottom-right positions of each bar
            let x = rect.min.x;

            for (i,v) in tv.iter().enumerate(){
                let bar_height = v * available_space.y;
                let bar_rect = Rect::from_min_size(
                    Pos2::new(x + 50f32*i as f32, rect.max.y - bar_height),
                    Vec2::new(20f32, bar_height),
                );
        
                // Fill the bar with a color
                painter.rect_filled(bar_rect, 0.0, Color32::from_rgb(100, 150, 250));
            }
        });

        ctx.request_repaint();
    }
}



fn f0_estimation(data: Vec<f32>) -> Vec<f32> {
    let sample_rate = 44100f32;
    // try 100 ~ 480 samples, or 100Hz to 480Hz
    let pitch = Vec::<f32>::from([130.8128, 146.8324, 164.8138, 174.6141, 195.9977, 220.0000, 246.9417]);
    let mut pitch_samples = Vec::<usize>::new();
    for p in pitch {
        pitch_samples.push((sample_rate / p) as usize);
    }
    
    let length = data.iter().count() - 1;
    let product_length:usize = length - pitch_samples[0];

    let base = &data[0..product_length];
    
    let mut result = Vec::<f32>::new();

    for i in pitch_samples {
        let shifted = &data[i..(i+product_length)];
        let mut r = 0f32;
        for j in 0..product_length {
            r = r + base[j] * shifted[j];
        }
        result.push(r);
    }

    result
}

fn main() -> eframe::Result {

    let shared_values = Arc::new(Mutex::new(vec![0f32; 7]));
    let clone = Arc::clone(&shared_values);


    std::thread::spawn(move || {
        let shared_values = clone;

        // Set up the audio host
        let host = cpal::default_host();

        // Get the default input device (usually the microphone)
        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        // Get the default input format
        let config = device.default_input_config().expect("Failed to get default input format");

        println!("channels = {}", config.channels());
        std::thread::sleep(Duration::from_millis(500));

        println!("Using input device: {:?}", device.name());
        println!("Input format: {:?}", config);

        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        // Create a stream to capture audio
        let stream = device
            .build_input_stream(
                &config.into(),
                {
                    let buffer = Arc::clone(&buffer);
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // This closure is called every time audio data is available
                        // `data` is a slice of audio samples from the microphone
                        // println!("Received audio data: {:?}", data);
                        // println!("Recieved audio data!");
                        let mut buf= buffer.lock().expect("the audio is lagging too much");

                        buf.extend_from_slice(data);
                    }
                },
                move |err| {
                    eprintln!("Error occurred on input stream: {:?}", err);
                },
                None
            )
            .expect("Failed to build input stream");

        // Start the stream
        stream.play().expect("Failed to start stream");

        // Keep doing the f0 estimation if there's two buffer samples
        
        loop {
            std::thread::sleep(std::time::Duration::from_millis(5));
            let mut data = buffer.lock().expect("aaaaa");

            if data.iter().count() >= 882 * 4 {
                
                let new_data = data.iter().enumerate().filter(|(x,_)| x % 2 == 0).map(|(_, y)| y.clone()).collect();
                let result = f0_estimation(new_data);
                println!("pitch = {:?}", result);
                let mut value = shared_values.lock().unwrap();
                *value = result;
                data.clear();
            };
        }
    } );

    // UI
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Autocorrelation Viewer",
        options,
        Box::new(|_cc| {
            Ok( Box::new(MyApp {
                values: shared_values,
            }) )
        }),
    )
}
