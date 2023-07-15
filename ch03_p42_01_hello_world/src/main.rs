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
    // ディスプレイドライバ(wio_terminal)
    let core::prelude::v1::Ok((mut display, _backlight)) = preset_display() else { todo!() };

    // 背景(embedded_graphics)
    let background = preset_background(&mut display);

    // Hello Worldの文字列
    // フォント設定
    use embedded_graphics::mono_font::MonoTextStyle;
    let style = MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        embedded_graphics::pixelcolor::Rgb565::WHITE,
    );
    // 文字列の描画
    let x: i32 = 10;
    let y: i32 = (DISPLAY_HEIGHT / 2u32) as i32;
    embedded_graphics::text::Text::new(
        "Hello Rust!\nI am surprised that file size rose drastically.",
        embedded_graphics::prelude::Point::new(x, y),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数

/// Wio Terminalのディスプレイドライバを初期化する
fn preset_display() -> Result<(wio_terminal::LCD, wio_terminal::aliases::LcdBacklight), ()> {
    let mut peripherals = wio_terminal::pac::Peripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
    let core = wio_terminal::pac::CorePeripherals::take().unwrap(); // 周辺機器(CPUコアとメモリ以外)
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
    sets.display.init(
        &mut clocks,
        peripherals.SERCOM7,
        &mut peripherals.MCLK,
        100_u32.mhz(),
        &mut delay,
    )
}

use embedded_graphics; // 描画関係
use embedded_graphics::prelude::*; // 役割が多いのでとりあえずインポートしておく

/// ディスプレイの背景を初期化する
fn preset_background(
    display: &mut wio_terminal::LCD,
) -> embedded_graphics::primitives::Styled<
    embedded_graphics::primitives::Rectangle,
    embedded_graphics::primitives::PrimitiveStyle<embedded_graphics::pixelcolor::Rgb565>,
> {
    // フォント等の設定
    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build(); // 塗りつぶしの設定

    // 背景は塗りつぶし長方形を用いる
    let background = embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::prelude::Point::new(0, 0),
        embedded_graphics::prelude::Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT),
    )
    .into_styled(style);

    // 塗りつぶし処理
    background.draw(display).unwrap();

    background
}
