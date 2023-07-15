//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio_terminal; // 必須クレート
use wio_terminal::prelude::*; // よく使うtraitやstructureが収められているようだ。ほぼ必須。
                              //const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
                              //const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

#[wio_terminal::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // 制御関係(wio_terminal)の初期化
    let mut peripherals = wio_terminal::pac::Peripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let sets = wio_terminal::Pins::new(peripherals.PORT).split(); // 入出力

    // 画面表示の描画間隔を設定するために時計インスタンス
    let mut clocks = wio_terminal::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    // 制御関係(wio_terminal)の初期化
    let mut core = wio_terminal::pac::CorePeripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    // 画面表示の描画間隔
    let mut delay = wio_terminal::hal::delay::Delay::new(core.SYST, &mut clocks);

    // LEDの初期化
    // 書籍の内容だとエラーになる
    // Githubの内容を参考にすること
    // https://github.com/atsamd-rs/atsamd/blob/0820f0df58eb8705ddfa6533ed76953d18e6b992/boards/wio_terminal/examples/blinky.rs
    let mut user_led = sets.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    // 組込みはloop必須
    loop {
        user_led.toggle().ok();
        delay.delay_ms(200u8);
    }
    // ここまでloop処理
}
// ここまでmain関数
