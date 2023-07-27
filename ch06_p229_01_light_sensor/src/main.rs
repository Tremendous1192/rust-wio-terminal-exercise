//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*; // ほぼ必須
use wio_terminal as wio; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

use eg::prelude::*; // 役割が多いのでとりあえずインポートしておく
use embedded_graphics as eg; // 描画関係

use itoa; // 整数型を文字列スライスに変換する

#[wio::entry]
fn main() -> ! {
    // 初期化
    // 必須
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let core = wio::pac::CorePeripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut sets = wio::Pins::new(peripherals.PORT).split();
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);

    // 光センサ読み取り用の ADC とピン
    let (mut light, mut pd1) =
        sets.light_sensor
            .init(peripherals.ADC1, &mut clocks, &mut peripherals.MCLK);

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

    // 背景
    let background = make_black_rectangle(0_i32, 0_i32, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    background.draw(&mut display).unwrap();

    // 前回の入力値を塗りつぶして見えなくする黒い四角形
    let refresher = make_black_rectangle(0_i32, 20_i32, 50_u32, 30_u32);

    // 説明文を表示する
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_10X20,
        eg::pixelcolor::Rgb565::WHITE,
    );
    eg::text::Text::new(
        "Input value of light senser",
        eg::prelude::Point::new(15_i32, 15_i32),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // 1秒ごとに光センサの入力値を画面に表示する
    loop {
        // 前回の入力値を塗りつぶす
        refresher.draw(&mut display).unwrap();

        // ADC入力をstr型に変換する
        let value: u16 = nb::block!(light.read(&mut pd1)).unwrap();
        let mut buffer = itoa::Buffer::new();
        let printed = buffer.format(value);

        // 光センサの入力値を画面に表示する
        eg::text::Text::new(printed, eg::prelude::Point::new(15_i32, 35_i32), style)
            .draw(&mut display)
            .unwrap();

        delay.delay_ms(1_000_u16);
    }
}

// 黒い四角形を作成する
fn make_black_rectangle(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> embedded_graphics::primitives::Styled<
    embedded_graphics::primitives::Rectangle,
    embedded_graphics::primitives::PrimitiveStyle<embedded_graphics::pixelcolor::Rgb565>,
> {
    let style = eg::primitives::PrimitiveStyleBuilder::new()
        .fill_color(eg::pixelcolor::Rgb565::BLACK)
        .build();
    let rectamgle = eg::primitives::Rectangle::new(
        eg::prelude::Point::new(x, y),
        eg::prelude::Size::new(width, height),
    )
    .into_styled(style);
    rectamgle
}

