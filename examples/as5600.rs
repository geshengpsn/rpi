fn main() {
    let i2c = rppal::i2c::I2c::new().unwrap();
    let mut as5600 = as5600::As5600::new(i2c);
    let config = as5600.config().unwrap();
    println!("{:?}", config);

    loop {
        let value = as5600.angle().unwrap();
        println!("{:?}", value);
    }
}