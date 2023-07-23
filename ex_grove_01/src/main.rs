//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*; // ほぼ必須
use wio_terminal as wio; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

#[wio_terminal::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // 必須
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let core = wio::pac::CorePeripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);
    // 内臓ディスプレイ等を簡単に扱う場合はsets, ベタに通信ピンを使う場合はpins
    //let mut sets = wio::Pins::new(peripherals.PORT).split();
    let mut pins = wio::Pins::new(peripherals.PORT);

    // 正面左側の Groveピンのインスタンス
    let mut i2c1_scl = pins.i2c1_scl.into_push_pull_output();
    i2c1_scl.set_low().unwrap();
    let mut i2c1_sda = pins.i2c1_sda.into_push_pull_output();
    i2c1_sda.set_low().unwrap();

    // ここまで 初期化

    // Grove-Buzzer を鳴らす(うるさいので100ミリ秒だけ)
    // sclが送信側のようだ
    delay.delay_ms(100u8);
    i2c1_scl.toggle().ok();
    delay.delay_ms(100u8);
    i2c1_scl.toggle().ok();

    // こちらは動かない
    // 受信側のようだ
    // 壊れたら嫌なので、コメントアウトのままにすること
    //delay.delay_ms(100u8);
    //i2c1_sda.toggle().ok();
    //delay.delay_ms(100u8);
    //i2c1_sda.toggle().ok();

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数
