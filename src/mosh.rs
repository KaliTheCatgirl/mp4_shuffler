use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use mp4::{
    AacConfig, AvcConfig, HevcConfig, MediaConfig, MediaType, Mp4Config, Result, TrackConfig,
    TtxtConfig, Vp9Config,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn mosh(src_filename: impl AsRef<Path>, dst_filename: impl AsRef<Path>, shuffle_start_fraction: f32) -> Result<()> {
    let src_file = File::open(src_filename)?;
    let size = src_file.metadata()?.len();
    let reader = BufReader::new(src_file);

    let dst_file = File::create(dst_filename)?;
    let writer = BufWriter::new(dst_file);

    let mut mp4_reader = mp4::Mp4Reader::read_header(reader, size)?;
    let mut mp4_writer = mp4::Mp4Writer::write_start(writer, &Mp4Config {
        major_brand: *mp4_reader.major_brand(),
        minor_version: mp4_reader.minor_version(),
        compatible_brands: mp4_reader.compatible_brands().to_vec(),
        timescale: mp4_reader.timescale(),
    })?;

    for track in mp4_reader.tracks().values() {
        let media_conf = match track.media_type()? {
            MediaType::H264 => MediaConfig::AvcConfig(AvcConfig {
                width: track.width(),
                height: track.height(),
                seq_param_set: track.sequence_parameter_set()?.to_vec(),
                pic_param_set: track.picture_parameter_set()?.to_vec(),
            }),
            MediaType::H265 => MediaConfig::HevcConfig(HevcConfig {
                width: track.width(),
                height: track.height(),
            }),
            MediaType::VP9 => MediaConfig::Vp9Config(Vp9Config {
                width: track.width(),
                height: track.height(),
            }),
            MediaType::AAC => MediaConfig::AacConfig(AacConfig {
                bitrate: track.bitrate(),
                profile: track.audio_profile()?,
                freq_index: track.sample_freq_index()?,
                chan_conf: track.channel_config()?,
            }),
            MediaType::TTXT => MediaConfig::TtxtConfig(TtxtConfig {}),
        };

        let track_conf = TrackConfig {
            track_type: track.track_type()?,
            timescale: track.timescale(),
            language: track.language().to_string(),
            media_conf,
        };

        mp4_writer.add_track(&track_conf)?;
    }

    for track_id in mp4_reader.tracks().keys().copied().collect::<Vec<u32>>() {
        let sample_count = mp4_reader.sample_count(track_id)?;

        // convert shuffle fraction to start offset, add 1 to always write first i-frame
        let starting_sample = ((sample_count as f32 * shuffle_start_fraction) as u32 + 1).min(sample_count);

        // write first samples in order
        for i in 0..starting_sample.min(sample_count) {
            let sample = mp4_reader.read_sample(track_id, i + 1)?.unwrap();
            mp4_writer.write_sample(track_id, &sample)?;
        }

        // shuffle everything else
        let mut sample_indices = (starting_sample..sample_count).collect::<Vec<u32>>();
        sample_indices.shuffle(&mut thread_rng());
        for sample_index in sample_indices {
            let sample = mp4_reader.read_sample(track_id, sample_index + 1)?.unwrap();
            mp4_writer.write_sample(track_id, &sample)?;
        }
    }

    mp4_writer.write_end()?;
    mp4_writer.into_writer().flush()?;

    Ok(())
}
