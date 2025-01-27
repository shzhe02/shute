use shute::{Instance, PowerPreference};

async fn check() {
    let instance = Instance::new();
    let performance_device = instance
        .autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Performance): {:#?}",
        performance_device.info()
    );
    let lowpower_device = instance
        .autoselect(PowerPreference::LowPower, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Low Power): {:#?}",
        lowpower_device.info()
    );
    let no_preference_device = instance
        .autoselect(PowerPreference::None, shute::LimitType::Highest)
        .await
        .unwrap();
    println!(
        "Autoselected Device (Performance): {:#?}",
        no_preference_device.info()
    );
}

fn main() {
    pollster::block_on(check());
}
