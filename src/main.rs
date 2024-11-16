use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{sync::{Arc, Mutex}, time::Duration};


struct MyApp {
    values: Vec<f32>
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            values: vec![0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32]
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });

            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
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
        let mut r:f32 = 0f32;
        for j in 0..product_length {
            r = r + base[j] * shifted[j];
        }
        result.push(r);
    }

    result
}

fn main() {
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
            std::thread::spawn(move || {
                // println!("\n\nDOING F0 ESTIMATION!!!!!");
                println!("pitch = {:?}", f0_estimation(new_data));
            });
            data.clear();
        };
    }

    // Keep the program running
    
}
