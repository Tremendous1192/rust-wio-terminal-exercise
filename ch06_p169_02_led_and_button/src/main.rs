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
    // 基本制御
    let mut peripherals = wio_terminal::pac::Peripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let sets = wio_terminal::Pins::new(peripherals.PORT).split(); // 入出力

    // LED(wio_terminal)の初期化
    let mut user_led = sets.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    // ボタン1の初期化
    let mut button1 = sets.buttons.button1.into_floating_input();

    // 組込みはloop必須
    loop {
        // ボタン1を押している場合、LEDを点灯する
        if button1.is_low().unwrap() {
            user_led.set_high().unwrap();
        }
        else {
            user_led.set_low().unwrap();
        }        
    }
    // ここまでloop処理
}
// ここまでmain関数
