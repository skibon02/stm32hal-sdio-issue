#![no_std]
#![no_main]


#![allow(unused_variables)]

use cortex_m_rt::entry;
use defmt::{error, info};

use stm32f4xx_hal::{
    pac,
    prelude::*,
    sdio::{ClockFreq, SdCard, Sdio},
};

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let device = unwrap!(pac::Peripherals::take());
    let core = unwrap!(cortex_m::Peripherals::take());

    let rcc = device.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .require_pll48clk()
        .sysclk(42.MHz())
        .hclk(42.MHz())
        .pclk1(42.MHz())
        .pclk2(42.MHz())
        .freeze();

    info!("Clock setup successful!");
    info!("Clocks result: {:?}", Debug2Format(&clocks));

    defmt::assert!(clocks.is_pll48clk_valid());

    info!("Clock setup successful 2!");

    let mut delay = core.SYST.delay(&clocks);

    let gpioc = device.GPIOC.split();
    let gpiod = device.GPIOD.split();

    let d0 = gpioc.pc8.internal_pull_up(true);
    let d1 = gpioc.pc9.internal_pull_up(true);
    let d2 = gpioc.pc10.internal_pull_up(true);
    let d3 = gpioc.pc11.internal_pull_up(true);
    let clk = gpioc.pc12;
    let cmd = gpiod.pd2.internal_pull_up(true);
    let mut sdio: Sdio<SdCard> = Sdio::new(device.SDIO, (clk, cmd, d0, d1, d2, d3), &clocks);

    info!("Waiting for card...");

    // Wait for card to be ready
    loop {
        match sdio.init(ClockFreq::F24Mhz) {
            Ok(_) => break,
            Err(_err) => (),
        }

        delay.delay_ms(1000);
    }

    let sts = sdio.read_sd_status().unwrap();
    info!("SD Status: {:?}", Debug2Format(&sts));

    let nblocks = sdio.card().map(|c| c.block_count()).unwrap_or(0);
    info!("Card detected: nbr of blocks: {:?}", nblocks);

    // Read a block from the card and print the data
    let mut block = [0u8; 512];

    info!("Read start...");
    match sdio.read_block(0, &mut block) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to read block: {:?}", Debug2Format(&err));
        }
    }
    info!("Read finished! First 10 bytes:");

    for b in block.iter().take(10) {
        defmt::println!("{:X} ", b);
    }

    info!("Write zeroes start...");
    match sdio.write_block(0, &[0; 512]) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to write block: {:?}", Debug2Format(&err));
        }
    }
    info!("Write zeroes finished!");

    info!("Read start...");
    match sdio.read_block(0, &mut block) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to read block: {:?}", Debug2Format(&err));
        }
    }

    info!("Read finished! First 10 bytes:");

    for b in block.iter().take(10) {
        defmt::println!("{:X} ", b);
    }

    let mut block = [0u8; 512];
    block[0..10].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34]);

    info!("Write random data start...");
    match sdio.write_block(0, &block) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to write block: {:?}", Debug2Format(&err));
        }
    }
    info!("Write random data finished!");

    info!("Clock test...");

    for i in 0..10 {
        cortex_m::asm::delay(clocks.sysclk().to_Hz() / 3 * 2);
        info!("Delay finished {}", i);
    }

    loop {
        continue;
    }
}