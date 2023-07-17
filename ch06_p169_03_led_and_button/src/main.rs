//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio_terminal as wio; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう
use wio::prelude::*; // ほぼ必須のクレート

// ボタン操作の最小構成
use cortex_m::interrupt::{free as disable_interrupts, CriticalSection};
use heapless::{consts::U8, spsc::Queue};
use wio::{button_interrupt, pac::interrupt, Button, ButtonController, ButtonEvent};
static mut BUTTON_CTRLR: Option<ButtonController> = None;
static mut Q: Queue<ButtonEvent, U8> = Queue(heapless::i::Queue::new());
button_interrupt!(
    BUTTON_CTRLR,
    unsafe fn on_button_event(_cs: &CriticalSection, event: ButtonEvent) {
        let mut q = Q.split().0;
        q.enqueue(event).ok();
    }
);
// ここまで ボタン操作の最小構成

#[wio::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // ほぼ必須の構成
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let mut core = wio::pac::CorePeripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);
    let sets = wio::Pins::new(peripherals.PORT).split();

    // ボタン操作
    let button_ctrlr = sets
        .buttons
        .init(peripherals.EIC, &mut clocks, &mut peripherals.MCLK);
    let nvic = &mut core.NVIC;
    disable_interrupts(|_| unsafe {
        button_ctrlr.enable(nvic);
        BUTTON_CTRLR = Some(button_ctrlr);
    });
    let mut consumer = unsafe { Q.split().1 };

    // 内臓LED
    let mut user_led = sets.user_led.into_push_pull_output();
    user_led.set_low().unwrap();
    let mut interval: u16 = 200_u16;
    // ここまで 初期化

    // 組込みはloop必須
    loop {
        // ボタン操作の処理を行う
        if let Some(press) = consumer.dequeue() {
            match press.button {
                Button::TopLeft => {
                    interval = 100_u16;
                }
                Button::TopMiddle => {
                    interval = 200_u16;
                }
                Button::Left => {
                    interval = 1000_u16;
                }
                Button::Right => {
                    interval = 100_u16;
                }
                Button::Down => {
                    interval = 200_u16;
                }
                Button::Up => {
                    interval = 2000_u16;
                }
                Button::Click => {
                    interval = 3000_u16;
                }
            }
        }

        // ボタン操作が働いていることを確認するためのLED点滅
        user_led.toggle().ok();
        delay.delay_ms(interval);
    }
    // ここまでloop処理
}
// ここまでmain関数
