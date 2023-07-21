//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio_terminal; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

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
            10_u32.mhz(), // 書籍やChatGPTのままだと型名をつけましょうと怒られる
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

    use embedded_graphics::mono_font::MonoTextStyle;
    let x: i32 = 10;
    let y: i32 = (DISPLAY_HEIGHT / 2u32) as i32;
    let style = MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        embedded_graphics::pixelcolor::Rgb565::WHITE,
    );
    embedded_graphics::text::Text::new(
        "Hello Rust!\nI am surprised that file size rose drastically.",
        embedded_graphics::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();
    // ここまで 描画関係(embedded_graphics)

    // 組込みはloop必須
    loop
    {}
    // ここまでloop処理
}
// ここまでmain関数

use embedded_graphics; // 描画関係
use embedded_graphics::prelude::*; // 役割が多いのでとりあえずインポートしておく
