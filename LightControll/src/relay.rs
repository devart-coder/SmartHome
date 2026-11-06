use esp_idf_hal::i2c::I2cDriver;

pub struct Relay {
    i2c: I2cDriver<'static>,
}
impl Relay {
    const WAVESHARE_ADDRESS: u8 = 0x20;
    const DIRECTION_ADDRESS: u8 = 0x03;
    const OUTPUT_ADDRESS: u8 = 0x01;
    pub fn new(mut i2c: I2cDriver<'static>) -> Self {
        i2c.write(Self::WAVESHARE_ADDRESS, &[Self::DIRECTION_ADDRESS, 0], 100)
            .unwrap();
        i2c.write(Self::WAVESHARE_ADDRESS, &[Self::OUTPUT_ADDRESS, 0], 100)
            .unwrap();
        Self { i2c }
    }
    pub fn modify<F>(&mut self, f: F)
    where
        F: Fn(RelayArrayBuilder) -> RelayArrayBuilder,
    {
        let mut current_array = [0u8; 1];
        self.i2c
            .write_read(
                Self::WAVESHARE_ADDRESS,
                &[Self::OUTPUT_ADDRESS],
                &mut current_array,
                100,
            )
            .unwrap();
        let builder = RelayArrayBuilder::new(current_array[0]);
        let array = f(builder).build();
        self.i2c
            .write(Self::WAVESHARE_ADDRESS, &[Self::OUTPUT_ADDRESS, array], 100)
            .unwrap();
    }
}

pub struct RelayArrayBuilder {
    value: u8,
}
impl RelayArrayBuilder {
    fn new(value: u8) -> Self {
        RelayArrayBuilder { value }
    }
    pub fn ch1(self) -> Channel {
        Channel::new(1, self.value)
    }
    pub fn ch2(self) -> Channel {
        Channel::new(2, self.value)
    }
    pub fn ch3(self) -> Channel {
        Channel::new(3, self.value)
    }
    pub fn ch4(self) -> Channel {
        Channel::new(4, self.value)
    }
    pub fn ch5(self) -> Channel {
        Channel::new(5, self.value)
    }
    pub fn ch6(self) -> Channel {
        Channel::new(6, self.value)
    }
    pub fn ch7(self) -> Channel {
        Channel::new(7, self.value)
    }
    pub fn ch8(self) -> Channel {
        Channel::new(8, self.value)
    }
    fn build(self) -> u8 {
        self.value
    }
}

pub struct Channel {
    index: u8,
    value: u8,
}
impl Channel {
    fn new(index: u8, value: u8) -> Self {
        Self { index, value }
    }
    pub fn turn_on(&mut self) -> RelayArrayBuilder {
        RelayArrayBuilder {
            value: self.value | (1 << (self.index - 1)),
        }
    }
    pub fn turn_off(&mut self) -> RelayArrayBuilder {
        RelayArrayBuilder {
            value: self.value & !(1 << (self.index - 1)),
        }
    }
    pub fn toggle(&mut self) -> RelayArrayBuilder {
        RelayArrayBuilder {
            value: self.value ^ (1 << (self.index - 1)),
        }
    }
}
