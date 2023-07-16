//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio_terminal; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅. ほぼ必須
const DISPLAY_HEIGHT: u32 = 240; // 画面高さ. ほぼ必須

use embedded_graphics; // 描画関係
use embedded_graphics::prelude::*; // 役割が多いのでとりあえずインポートしておく

#[wio_terminal::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // 制御関係(wio_terminal)
    let mut peripherals = wio_terminal::pac::Peripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let mut core = wio_terminal::pac::CorePeripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let sets = wio_terminal::Pins::new(peripherals.PORT).split(); // 入出力

    // 画面表示の描画間隔を設定するために時計インスタンス
    let mut clocks = wio_terminal::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    // 画面表示の描画間隔
    let mut delay = wio_terminal::hal::delay::Delay::new(core.SYST, &mut clocks);

    // ディスプレイドライバ
    use wio_terminal::prelude::_atsamd21_hal_time_U32Ext;
    let (mut display, _backlight) = sets
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM7,
            &mut peripherals.MCLK,
            1000_u32.mhz(), // 書籍やChatGPTのままだと型名をつけましょうと怒られる
            &mut delay,
        )
        .unwrap();
    // ここまで 制御関係(wio_terminal)

    // 描画関係(embedded_graphics)
    // 描画設定
    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build(); // 塗りつぶしの設定
                  // 背景の描画
    let background = embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::prelude::Point::new(0, 0),
        embedded_graphics::prelude::Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT),
    )
    .into_styled(style); // 背景の設定
    background.draw(&mut display).unwrap();

    // embedded-graphics-simulatorのインポートが上手くいかなかったので、使用しない
    // ドキュメントの作例をコピペしました
    // https://docs.rs/embedded-graphics/latest/embedded_graphics/primitives/line/struct.Line.html
    // Red 1 pixel wide line from (50, 20) to (60, 35)
    embedded_graphics::primitives::Line::new(embedded_graphics::prelude::Point::new(50, 20), embedded_graphics::prelude::Point::new(60, 35))
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(embedded_graphics::pixelcolor::Rgb565::RED, 1))
        .draw(&mut display).unwrap();

    // Green 10 pixel wide line with translation applied
    embedded_graphics::primitives::Line::new(embedded_graphics::prelude::Point::new(50, 20), embedded_graphics::prelude::Point::new(60, 35))
        .translate(embedded_graphics::prelude::Point::new(-30, 10))
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(embedded_graphics::pixelcolor::Rgb565::GREEN, 10))
        .draw(&mut display).unwrap();
    // ここまで 描画関係(embedded_graphics)

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数
