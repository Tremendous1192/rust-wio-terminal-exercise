//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio_terminal as wio; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

use embedded_graphics; // 描画関係
use embedded_graphics::prelude::*; // 役割が多いのでとりあえずインポートしておく
use micromath::F32Ext; // f32型の関数

use wio::pac::interrupt;
use wio::prelude::_atsamd21_hal_time_U32Ext; // マイクの集音機構の独立化
use wio::prelude::*;

// main() 関数とADCの割り込みハンドラで共有するリソース
struct Ctx {
    adc: wio::hal::adc::InterruptAdc<wio::pac::ADC1, wio::hal::adc::FreeRunning>,
    buffers: [SamplingBuffer; 2], // ADC結果のバッファ2面分
    // 現在ADC結果取り込み先のバッファへの参照
    sampling_buffer: Option<&'static mut SamplingBuffer>,
    // 現在信号処理中のバッファへの参照
    processing_buffer: Option<&'static mut SamplingBuffer>,
}
static mut CTX: Option<Ctx> = None;
type SamplingBuffer = heapless::Vec<f32, 256_usize>; //サンプリングバッファの型

// FFT のパラメータ
const AVERAGING_FACTOR: u32 = 4; // 平均化フィルタのサンプル点数
const FFT_POINTS: usize = 256; // FFTをするサンプル点数
const ADC_SAMPLING_RATE: f32 = 83333.0; // ADCのサンプリングレート
#[allow(dead_code)]
// 平均化フィルタ後のサンプリングレート
const SAMPLING_RATE: f32 = ADC_SAMPLING_RATE / AVERAGING_FACTOR as f32;
const AMPLITUDE: f32 = 4096.0; // サンプル値の最大振幅

// C言語のライブラリを呼び出す関数
#[no_mangle]
fn fminf(a: f32, b: f32) -> f32 {
    match a.partial_cmp(&b) {
        None => a,
        Some(core::cmp::Ordering::Less) => a,
        Some(core::cmp::Ordering::Equal) => a,
        Some(core::cmp::Ordering::Greater) => b,
    }
}
#[no_mangle]
fn fmaxf(a: f32, b: f32) -> f32 {
    match a.partial_cmp(&b) {
        None => a,
        Some(core::cmp::Ordering::Less) => b,
        Some(core::cmp::Ordering::Equal) => b,
        Some(core::cmp::Ordering::Greater) => a,
    }
}

#[interrupt]
fn ADC1_RESRDY() {
    // データをサンプリングし、`sampling_buffer`を埋める。
    static mut AVERAGE: f32 = 0.0; // 平均値
    static mut AVERAGE_COUNT: u32 = 0; // 平均値計算時のサンプル数カウント
    unsafe {
        let ctx = CTX.as_mut().unwrap();
        if let Some(sample) = ctx.adc.service_interrupt_ready() {
            // サンプルデータがあれば平均値計算のために積算する
            *AVERAGE += sample as f32;
            *AVERAGE_COUNT += 1_u32;
            if *AVERAGE_COUNT == AVERAGING_FACTOR {
                //平均値計算回数文のサンプルデータを積算した
                let sampling_buffer = ctx.sampling_buffer.as_mut().unwrap();
                if sampling_buffer.len() == sampling_buffer.capacity() {
                    // サンプリングバッファがいっぱいなので処理用バッファが空
                    // つまり処理が終わっているなら入れ替える
                    if ctx.processing_buffer.as_mut().unwrap().len() == 0 {
                        core::mem::swap(&mut ctx.processing_buffer, &mut ctx.sampling_buffer);
                    }
                } else {
                    //サンプリングバッファに平均値を追加する
                    let _ = sampling_buffer.push(*AVERAGE / (AVERAGING_FACTOR as f32));
                }
                // 積算カウントを0に戻す
                *AVERAGE_COUNT = 0;
                *AVERAGE = 0_f32;
            }
        }
    }
}

// 長方形を描くマクロ
macro_rules! draw_rectangle {
    ($style:expr,&mut $display:expr,$x_y_width_height:expr) => {
        let rectangle = crate::embedded_graphics::primitives::Rectangle::new(
            crate::embedded_graphics::prelude::Point::new(
                $x_y_width_height.0.try_into().unwrap(),
                $x_y_width_height.1.try_into().unwrap(),
            ),
            crate::embedded_graphics::prelude::Size::new(
                $x_y_width_height.2.try_into().unwrap(),
                $x_y_width_height.3.try_into().unwrap(),
            ),
        )
        .into_styled($style);

        rectangle.draw(&mut $display).unwrap();
    };
}


#[wio::entry] // 必須アトリビュート
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

    // フリーランニングモードでADCを動かすようにInterruptAdc型を構築する
    let (microphone_adc, mut microphone_pin) =
        sets.microphone
            .init(peripherals.ADC1, &mut clocks, &mut peripherals.MCLK);
    let mut microphone_adc: wio::hal::adc::InterruptAdc<_, wio::hal::adc::FreeRunning> =
        wio::hal::adc::InterruptAdc::from(microphone_adc);
    // ADCの変換処理を開始する
    microphone_adc.start_conversion(&mut microphone_pin);

    // 共有リソースを初期化する
    unsafe {
        CTX = Some(Ctx {
            adc: microphone_adc,
            buffers: [heapless::Vec::new(), heapless::Vec::new()],
            sampling_buffer: None,
            processing_buffer: None,
        });
        // 2面分のサンプリングバッファを取り込み用途処理用にそれぞれ割り当てる
        let mut ctx = CTX.as_mut().unwrap();
        let (first, rest) = ctx.buffers.split_first_mut().unwrap();
        ctx.sampling_buffer = Some(first);
        ctx.processing_buffer = Some(&mut rest[0]);
    }

    // デバッグ用UARTを初期化する
    let mut serial = sets.uart.init(
        &mut clocks,
        wio::hal::time::Hertz(115_200u32),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
    );

    // ADC変換完了割り込み(RESRDY)を有効にしてサンプリングを開始する
    //writeln!(&mut serial, "start").unwrap();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(wio::pac::interrupt::ADC1_RESRDY);
    }

    // 再始動ボタンがbutton1, 一時停止ボタンがbutton2
    let button_restart = sets.buttons.button1.into_floating_input();
    let button_stop = sets.buttons.button2.into_floating_input();

    // FFTの窓関数としてHann窓を使うので係数を計算しておく
    // 振幅の正規化用に最大振幅で割っておく
    let mut hann_factor = [0_f32; FFT_POINTS];
    for i in 0..FFT_POINTS {
        use core::f32::consts::PI;
        hann_factor[i] =
            0.5_f32 * (1_f32 - (PI * 2.0_f32 * i as f32 / FFT_POINTS as f32).cos()) / AMPLITUDE;
    }
    let hann_factor = hann_factor;

    // 画面を初期化する
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

    // 画面のスペクトラム表示領域の内容を消す
    const SCREEN_WIDTH: i32 = 320;
    const SCREEN_HEIGHT: i32 = 240;

    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();
    draw_rectangle!(
        style,
        &mut display,
        (0_i32, 0_i32, DISPLAY_WIDTH, DISPLAY_HEIGHT)
    );

    // 描画設定
    const BAR_WIDTH: i32 = 2;
    const REAL_POINTS: usize = FFT_POINTS / 2;
    const NUMBER_OF_BARS: usize = REAL_POINTS;
    const DRAW_AREA_WIDTH: i32 = BAR_WIDTH * (NUMBER_OF_BARS as i32 + 1);
    let mut prev_bar_position = [0u8; NUMBER_OF_BARS as usize];
    // 一時停止のリクエスト
    let mut stop_req = false;
    let mut stop_ack = false;
    // ここまで 初期化

    // 組込みはloop必須
    loop {
        // 停止ボタンが押されていたら、停止要求をする
        if button_stop.is_low().unwrap() {
            stop_req = true;
        }
        // safe: processing_buffer は、mainループでclearするまで
        // 割り込みハンドラが触らないので注意
        let processing_buffer = unsafe {
            let ctx = CTX.as_mut().unwrap();
            ctx.processing_buffer.as_mut().unwrap()
        };
        let len = processing_buffer.len();
        let cap = processing_buffer.capacity();

        // 処理対象バッファにFFT点数分のサンプルデータがはいっている?
        if len == cap {
            for i in 0..FFT_POINTS {
                processing_buffer[i] *= hann_factor[i];
            }
            // 実部のみの入力に対する256点FFTを実行する
            let result = microfft::real::rfft_256(processing_buffer.as_mut());

            // スペクトルを描画する
            let offset_top: i32 = 0;
            let offset_left: i32 = (SCREEN_WIDTH - DRAW_AREA_WIDTH) / 2_i32;
            let area_height: i32 = SCREEN_HEIGHT;
            for (step, spectrum) in result.iter().enumerate() {
                // パワーの計算
                let power = spectrum.norm_sqr() / ((FFT_POINTS * FFT_POINTS) as f32);
                // 対数にする
                let relative_power = if power <= 0_f32 {
                    core::f32::NEG_INFINITY
                } else {
                    power.log10() * 10_f32
                };
                // 値からY座標を計算
                let height: i32 = -(((relative_power + 50_f32) * 5_f32)
                    .round()
                    .max(-area_height as f32)
                    .min(0.0) as i32);
                // Y座標から色を計算
                let intensity: i32 = (height * 255_i32) / (SCREEN_HEIGHT / 2_i32);
                let red = if height < SCREEN_HEIGHT / 2_i32 {
                    255_i32 - intensity
                } else {
                    0_i32
                };
                let green = if height < SCREEN_HEIGHT / 2_i32 {
                    intensity
                } else {
                    511_i32 - intensity
                };
                let blue = if height < SCREEN_HEIGHT / 2_i32 {
                    0_i32
                } else {
                    intensity - 256_i32
                };

                // 前回のバーを消す
                let start_x: i32 = offset_left + step as i32 * BAR_WIDTH;
                let end_x: i32 = offset_left + (step + 1_usize) as i32 * BAR_WIDTH;
                let prev_y = prev_bar_position[step] as i32;

                let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                    .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
                    .build();

                draw_rectangle!(style, &mut display, (start_x, prev_y, BAR_WIDTH, BAR_WIDTH));

                // 今回のバーを描く
                if stop_req {
                    // 停止要求時は見やすくするために棒グラフにする
                    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                        .fill_color(
                            embedded_graphics::pixelcolor::Rgb888::new(
                                red as u8,
                                green as u8,
                                blue as u8,
                            )
                            .into(),
                        )
                        .build();

                    draw_rectangle!(
                        style,
                        &mut display,
                        (start_x, offset_top + height, BAR_WIDTH, area_height - 1_i32)
                    );
                } else {
                    // 普段は棒グラフを描くのは遅いので点だけ描く
                    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                        .fill_color(
                            embedded_graphics::pixelcolor::Rgb888::new(
                                red as u8,
                                green as u8,
                                blue as u8,
                            )
                            .into(),
                        )
                        .build();

                    draw_rectangle!(
                        style,
                        &mut display,
                        (start_x, offset_top + height, BAR_WIDTH, BAR_WIDTH)
                    );
                }
                prev_bar_position[step] = (offset_top + height) as u8;
            }

            // 処理が終わったので処理用バッファをクリアする
            processing_buffer.clear();
            // 停止要求が着ていたら、処理していたことを通知
            stop_ack = stop_req;
        }

        if stop_ack {
            // 停止要求に対する処理が完了したので
            // リスタートボタンが押されるまで停止
            stop_req = false;
            stop_ack = false;
            while !button_restart.is_low().unwrap() {}
            // 画面クリア
            let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
                .build();
            draw_rectangle!(
                style,
                &mut display,
                (0_i32, 0_i32, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            );
        }
    }
    // ここまでloop処理
}
// ここまでmain関数
