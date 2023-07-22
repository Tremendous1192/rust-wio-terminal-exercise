//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*;
use wio_terminal as wio; // 必須クレート // ほぼ必須
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

#[wio::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // 必須インスタンス
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
    let mut sets = wio::Pins::new(peripherals.PORT).split();

    // ブザー（PWM）ドライバオブジェクト
    let mut buzzer = sets
        .buzzer
        .init(&mut clocks, peripherals.TCC0, &mut peripherals.MCLK);
    //           ド   レ    ミ   ファ  ソ   ラ   シ   ド
    let freqs = [261, 294, 329, 349, 392, 440, 494, 523];
    // デューティ比を50%に設定する
    // 周期的な現象において、"ある期間" に占める "その期間で現象が継続される期間" の割合
    // オーバーヒート対策
    let max_duty = buzzer.get_max_duty();
    buzzer.set_duty(wio::hal::pwm::Channel::_4, max_duty / 2);

    // 組込みはloop必須
    loop {
        // ドレミの音を1秒ずつ鳴らす
        for freq in freqs.iter() {
            // 周期（周波数）を設定する
            buzzer.set_period(freq.hz());

            // 1秒鳴らして止める
            buzzer.enable(wio::hal::pwm::Channel::_4);
            delay.delay_ms(1000u16);
            buzzer.disable(wio::hal::pwm::Channel::_4);
        }
    }
    // ここまでloop処理
}
// ここまでmain関数
