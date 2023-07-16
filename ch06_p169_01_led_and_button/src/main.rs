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

    // 時計
    let mut clocks = wio_terminal::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    // 遅延処理
    let mut core = wio_terminal::pac::CorePeripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let mut delay = wio_terminal::hal::delay::Delay::new(core.SYST, &mut clocks);// 画面表示の描画間隔

    // LED(wio_terminal)の初期化
    let mut user_led = sets.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    // ボタン(wio_terminal)の初期化
    let button_ctrlr = sets
        .buttons
        .init(peripherals.EIC, &mut clocks, &mut peripherals.MCLK);
    // よくわからない初期化
    let nvic = &mut core.NVIC;
    // メモリ解法か?
    disable_interrupts(|_| unsafe {
        button_ctrlr.enable(nvic);
        BUTTON_CTRLR = Some(button_ctrlr);
    });

    let mut consumer = unsafe { Q.split().1 };

    // 組込みはloop必須
    loop {
        if let Some(press) = consumer.dequeue() {
            match press.button {
                wio_terminal::Button::TopLeft => {
                    user_led.toggle().ok();
                }
                _ => {}
            }
        } else {
        }
    }
    // ここまでloop処理
}
// ここまでmain関数

// ↓ボタン処理のためのあれこれらしい
static mut BUTTON_CTRLR: Option<wio_terminal::ButtonController> = None;
static mut Q: heapless::spsc::Queue<wio_terminal::ButtonEvent, heapless::consts::U8> =
    heapless::spsc::Queue(heapless::i::Queue::new());

use cortex_m::interrupt::{free as disable_interrupts, CriticalSection};
use wio_terminal::wifi_prelude::*;
use wio_terminal::{button_interrupt, ButtonEvent};
button_interrupt!(
    BUTTON_CTRLR,
    unsafe fn on_button_event(_cs: &CriticalSection, event: ButtonEvent) {
        let mut q = Q.split().0;
        q.enqueue(event).ok();
    }
);
