use binrw::io::Cursor;
use binrw::BinWriterExt;
use std::io::{Read, Write};
use std::panic;

use binrw::{binrw, BinRead, BinWrite};
use log::{error, info, trace};
use modular_bitfield::prelude::*;

use psimple::Simple;
use pulse::context::introspect::SampleInfo;
use pulse::error::{Code, PAErr};
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;

/*
 * Data is transferred in UDP frames with a payload size of max. 1157 bytes, consisting of 5 bytes header and 1152 bytes PCM data. The latter number is divisible by 4, 6 and 8, so a full number of samples for all channels will always fit into a packet. The first header byte denotes the sampling rate. Bit 7 specifies the base rate: 0 for 48kHz, 1 for 44,1kHz. Other bits specify the multiplier for the base rate. The second header byte denotes the sampling width, in bits. The third header byte denotes the number of channels being transferred. The fourth and fifth header bytes make up the DWORD dwChannelMask from Microsofts WAVEFORMATEXTENSIBLE structure, describing the mapping of channels to speaker positions.
 */

struct SamplingRateHeader {
    multiplier: u32,
    base_rate: u32,
}

fn from_rate(rate: u32) -> SamplingRateHeader {
    let base_rate = if rate >= 48000 { 48000 } else { 44100 };
    let base_rate_bit = if base_rate == 44100 { 1 } else { 0 };
    let multiplier = rate / base_rate;

    SamplingRateHeader {
        multiplier: multiplier,
        base_rate: base_rate_bit,
    }
}

#[bitfield]
#[derive(BinRead, BinWrite, Clone, Copy, Debug)]
#[br(map = Self::from_bytes)]
#[bw(map = |&x| Self::into_bytes(x))]
struct ScreamHeader {
    sampling_rate_multiplier: B7,
    sampling_rate_base: B1,
    sampling_width: u8,
    channels: u8,
    unused_microsoft_api_shiz: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    info!("amongus");

    let SILENCE_FRAME_THRESHOLD = 2000;
    let output_address_osstr = std::env::var_os("ADDR").expect("address is required");
    let output_address = output_address_osstr
        .to_str()
        .expect("address should be utf8");
    info!("address: {:?}", output_address);

    let spec = Spec {
        format: Format::S32NE,
        channels: 2,
        rate: 48000,
    };
    assert!(spec.is_valid());

    let s = match Simple::new(
        None,
        "hellish_screams",
        Direction::Record,
        Some("0"),
        "record",
        &spec,
        None,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("fucked inittializing simple stream {:?}", e.to_string());
            std::panic!("fuck");
        }
    };
    let header_rate = from_rate(spec.rate);

    let header = ScreamHeader::new()
        .with_sampling_rate_multiplier(header_rate.multiplier.try_into().unwrap())
        .with_sampling_rate_base(header_rate.base_rate.try_into().unwrap())
        .with_sampling_width(32)
        .with_channels(2)
        .with_unused_microsoft_api_shiz(0);

    info!("header: {:?}", header);

    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;

    let mut silence_frames = 0;

    loop {
        let mut pcm_samples: [u8; 1152] = [0; 1152];
        let mut output: binrw::io::Cursor<Vec<u8>> = binrw::io::Cursor::new(Vec::new());
        match s.read(&mut pcm_samples) {
            Ok(()) => {
                let mut is_silence_frame: bool = true;
                for sample in pcm_samples {
                    if sample != 0 {
                        is_silence_frame = false;
                    }
                }

                let mut should_send: bool = false;
                if !is_silence_frame {
                    should_send = true;
                    silence_frames = 0;
                } else {
                    silence_frames += 1;
                    if silence_frames < SILENCE_FRAME_THRESHOLD {
                        should_send = true;
                    }
                }

                trace!(
                    "is silence? {:?} silence frames? {:?} should send {:?}",
                    is_silence_frame,
                    silence_frames,
                    should_send
                );

                if should_send {
                    trace!(
                        "sending data, first bytes are {:?} {:?} {:?} {:?}",
                        pcm_samples.get(0).unwrap(),
                        pcm_samples.get(1).unwrap(),
                        pcm_samples.get(2).unwrap(),
                        pcm_samples.get(3).unwrap()
                    );

                    output.write_le(&header)?;
                    output.write(&pcm_samples)?;

                    socket.send_to(&output.into_inner(), output_address)?;
                }
            }
            Err(e) => {
                error!("shit {:?}", e.to_string());
                std::panic!("fuck");
            }
        }
    }
}
