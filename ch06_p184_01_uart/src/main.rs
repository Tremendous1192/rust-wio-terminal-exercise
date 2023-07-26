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
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut sets: wio::Sets = wio::Pins::new(peripherals.PORT).split();

    // UARTドライバオブジェクトを初期化する
    // pub type HalUart = Uart<Config<UartPads>, Duplex>
    // この Uart は atsamd_hal::sercom::uart::Uart 構造体のこと
    // https://docs.rs/atsamd-hal/0.15.1/atsamd_hal/sercom/uart/struct.Uart.html
    let mut serial = sets.uart.init(
        &mut clocks,
        9600.hz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
    );
    // ここまで 初期化

    // Tera Term に「hello world」と出力する
    for c in b"hello world\n".iter() {
        // impl<C, D> Write<<C as AnyConfig>::Word> for Uart<C, D>
        // fn write(&mut self, word: C::Word) -> Result<(), Self::Error>
        nb::block!(serial.write(*c)).unwrap();
    }
    // データの読み込みメソッド
    // impl<C, D> Read<<C as AnyConfig>::Word> for Uart<C, D>
    // fn read(&mut self) -> Result<C::Word, Error>

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数
