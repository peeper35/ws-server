use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Seek, SeekFrom};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ZerodhaMessage {
    pub instrument_token: i32,
    pub last_price: f64,
    pub average_price: Option<f64>,
    pub last_quantity: Option<f64>,
    pub last_trade_time: Option<u64>,
    pub oi: Option<f64>,
    pub oi_day_high: Option<f64>,
    pub oi_day_low: Option<f64>,
    pub change: f64,
    pub volume: Option<u64>,
    pub ohlc: OHLC,
    pub buy: Option<Vec<BuySell>>,
    pub buy_quantity: Option<f64>,
    pub sell: Option<Vec<BuySell>>,
    pub sell_quantity: Option<f64>,
    pub timestamp: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuySell {
    pub orders: f64,
    pub price: f64,
    pub quantity: f64,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct OHLC {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl ZerodhaMessage {
    #[instrument(fields(zerodha = true))]
    pub fn parse(msg: &[u8]) -> Vec<ZerodhaMessage> {
        let mut tick_data = Vec::new();

        // Create a cursor to read from the byte buffer
        let mut reader = Cursor::new(msg);
        let number_of_packets = reader.read_i16::<BigEndian>().unwrap();

        let mut j: u64 = 2;

        for _ in 0..number_of_packets {
            let packet_length = reader.read_i16::<BigEndian>().unwrap();
            let instrument_token = reader.read_i32::<BigEndian>().unwrap();
            let divisor: f64 = 100.0;

            match packet_length {
                // Index quote/full
                32 => {
                    let last_price = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let high = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let low = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let open = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let close = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let change = ((last_price - close) * 100 as f64) / close;
                    let timestamp = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;

                    let zerodha_message = ZerodhaMessage {
                        instrument_token,
                        last_price,
                        ohlc: OHLC {
                            high,
                            low,
                            open,
                            close,
                        },
                        change,
                        timestamp: timestamp as u64,
                        ..Default::default()
                    };

                    tick_data.push(zerodha_message);
                }

                // Quote/Full
                184 => {
                    let last_price = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let last_quantity = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let average_price = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let volume = reader.read_i32::<BigEndian>().unwrap() as u64;
                    let buy_quantity = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let sell_quantity = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let open = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let high = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let low = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let close = reader.read_i32::<BigEndian>().unwrap() as f64 / divisor;
                    let change = ((last_price - close) * 100 as f64) / close;
                    let last_trade_time = reader.read_i32::<BigEndian>().unwrap() as u64;
                    let oi = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let oi_day_high = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let oi_day_low = reader.read_i32::<BigEndian>().unwrap() as f64;
                    let timestamp = reader.read_i32::<BigEndian>().unwrap() as f64;

                    let mut buy_depth_data = Vec::new();
                    let mut sell_depth_data = Vec::new();

                    for index in 0..10 {
                        let depth_data = BuySell {
                            quantity: reader.read_i32::<BigEndian>().unwrap() as f64,
                            price: reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                            orders: reader.read_i16::<BigEndian>().unwrap() as f64,
                        };

                        if index < 5 {
                            buy_depth_data.push(depth_data);
                        } else {
                            sell_depth_data.push(depth_data);
                        }

                        // Skip padding
                        reader.read_i16::<BigEndian>().unwrap();
                    }

                    let zerodha_message = ZerodhaMessage {
                        instrument_token,
                        last_price,
                        ohlc: OHLC {
                            high,
                            low,
                            open,
                            close,
                        },
                        change,
                        timestamp: timestamp as u64,
                        average_price: Some(average_price),
                        buy: Some(buy_depth_data),
                        buy_quantity: Some(buy_quantity),
                        last_quantity: Some(last_quantity),
                        last_trade_time: Some(last_trade_time),
                        oi: Some(oi),
                        oi_day_high: Some(oi_day_high),
                        oi_day_low: Some(oi_day_low),
                        sell: Some(sell_depth_data),
                        sell_quantity: Some(sell_quantity),
                        volume: Some(volume),
                    };

                    tick_data.push(zerodha_message);
                }

                _ => {
                    warn!("Undefined packet length received: {}", packet_length);
                }
            }

            // Adjust the reader's position after the current packet
            reader
                .seek(SeekFrom::Start(j + 2 + packet_length as u64))
                .unwrap();
            j = j + 2 + packet_length as u64;
        }

        tick_data
    }
}
