//! Quick example for seeing what devices are returned through the `Instance::autoselect` method
//! with all three power preferences, as well as all the devices returned through `Instance::devices`.

use shute::{Instance, PowerPreference};

async fn check() {
    let instance = Instance::new();
    println!("All devices:");
    for device in instance.devices() {
        if let Ok(device) = device {
            println!("{:#?}", device.info());
        }
    }
    println!("=====");
    let performance_device = instance
        .autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Performance): {:#?}",
        performance_device.info()
    );
    println!("Limits: {:#?}", performance_device.limits());
    println!("=====");
    let lowpower_device = instance
        .autoselect(PowerPreference::LowPower, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Low Power): {:#?}",
        lowpower_device.info()
    );
    println!("Limits: {:#?}", lowpower_device.limits());
    println!("=====");
    let no_preference_device = instance
        .autoselect(PowerPreference::None, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Performance): {:#?}",
        no_preference_device.info()
    );
    println!("Limits: {:#?}", no_preference_device.limits());
}

fn main() {
    pollster::block_on(check());
}
