//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*;
use wio_terminal as wio; // 必須クレート // ほぼ必須のモジュール
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

use eg::prelude::*;
use embedded_graphics as eg; // 描画関係 // 役割が多いのでとりあえずインポートしておく

#[wio_terminal::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    // 制御関係(wio_terminal)
    let mut peripherals = wio::pac::Peripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let mut core = wio::pac::CorePeripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let sets = wio::Pins::new(peripherals.PORT).split(); // 入出力

    // 画面表示の描画間隔を設定するために時計インスタンス
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    // 画面表示の描画間隔
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);

    // ディスプレイドライバ
    let (mut display, _backlight) = sets
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM7,
            &mut peripherals.MCLK,
            60_u32.mhz(),
            &mut delay,
        )
        .unwrap();
    // ここまで 制御関係(wio_terminal)

    // 描画関係(embedded_graphics)
    // 描画設定
    let style = eg::primitives::PrimitiveStyleBuilder::new()
        .fill_color(eg::pixelcolor::Rgb565::BLACK)
        .build(); // 塗りつぶしの設定
                  // 背景の描画
    let background = eg::primitives::Rectangle::new(
        eg::prelude::Point::new(0, 0),
        eg::prelude::Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT),
    )
    .into_styled(style); // 背景の設定
    background.draw(&mut display).unwrap();

    // 視認性の良いフォント
    // * FONT_10X20 大きい文字
    // * FONT_9X15_BOLD 太い文字
    // 視認性の良い色
    // * WHITE
    // * GREEN
    // * YELLOW
    // * CYAN

    //use eg::mono_font::MonoTextStyle;
    // FONT_10X20 視認性ヨシ
    let x: i32 = 15;
    let y: i32 = 15;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_10X20,
        eg::pixelcolor::Rgb565::WHITE,
    );
    eg::text::Text::new("FONT_10X20, WHITE, y = 15", eg::prelude::Point::new(x, y), style)
        .draw(&mut display)
        .unwrap();

    // FONT_9X18_BOLD
    let y: i32 = 30;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_9X18_BOLD,
        eg::pixelcolor::Rgb565::RED,
    );
    eg::text::Text::new(
        "FONT_9X18_BOLD, RED, y = 30",
        eg::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // FONT_9X18
    // GREEN 視認性ヨシ
    let y: i32 = 45;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_9X18,
        eg::pixelcolor::Rgb565::GREEN,
    );
    eg::text::Text::new("FONT_9X18, GREEN, y = 45", eg::prelude::Point::new(x, y), style)
        .draw(&mut display)
        .unwrap();

    // FONT_9X15_BOLD 視認性ヨシ
    let y: i32 = 60;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_9X15_BOLD,
        eg::pixelcolor::Rgb565::BLUE,
    );
    eg::text::Text::new(
        "FONT_9X15_BOLD, BLUE, y = 45",
        eg::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // FONT_9X15
    // YELLOW 視認性ヨシ
    let y: i32 = 75;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_9X15,
        eg::pixelcolor::Rgb565::YELLOW,
    );
    eg::text::Text::new("FONT_9X15, YELLOW, y = 75", eg::prelude::Point::new(x, y), style)
        .draw(&mut display)
        .unwrap();

    // FONT_8X13_ITALIC
    let y: i32 = 90;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_8X13_ITALIC,
        eg::pixelcolor::Rgb565::MAGENTA,
    );
    eg::text::Text::new(
        "FONT_8X13_ITALIC, MAGENTA, y = 90",
        eg::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // FONT_8X13_BOLD
    // CYAN 視認性ヨシ
    let y: i32 = 105;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_8X13_BOLD,
        eg::pixelcolor::Rgb565::CYAN,
    );
    eg::text::Text::new(
        "FONT_8X13_BOLD, CYAN, y = 105",
        eg::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // FONT_8X13
    // new(123_u8, _123_u8, 123_u8)
    let y: i32 = 120;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_8X13,
        eg::pixelcolor::Rgb565::new(123_u8, 123_u8, 123_u8),
    );
    eg::text::Text::new("FONT_8X13, new(123_u8, _123_u8, 123_u8), y = 120", eg::prelude::Point::new(x, y), style)
        .draw(&mut display)
        .unwrap();


            //use eg::mono_font::MonoTextStyle;
    // FONT_10X20 視認性ヨシ
    let y: i32 = 135;
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_10X20,
        eg::pixelcolor::Rgb565::WHITE,
    );
    eg::text::Text::new("15 pixels required\nin the margins, y = 135", eg::prelude::Point::new(x, y), style)
        .draw(&mut display)
        .unwrap();

    // ここまで 描画関係(embedded_graphics)

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数
