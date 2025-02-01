#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{Buf, Bytes};
pub use iterator::BlockIterator;

use crate::key::KeyVec;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the tutorial
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = Vec::with_capacity(
            self.data.len() + // actual data
            self.offsets.len() * std::mem::size_of::<u16>() + // offset array (u16)
            std::mem::size_of::<u16>(), // offset array length (u16)
        );

        // Write data
        buf.extend_from_slice(&self.data);

        // Write offsets
        for offset in &self.offsets {
            buf.extend_from_slice(&offset.to_le_bytes());
        }

        // Write offset array length
        buf.extend_from_slice(&(self.offsets.len() as u16).to_le_bytes());

        Bytes::from(buf)
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        let u16_bytes = std::mem::size_of::<u16>();

        // 检查数据长度至少包含偏移量数组长度
        if data.len() < u16_bytes {
            panic!("Invalid data: too short to contain offset array length");
        }

        // 读取偏移量数组长度
        let last_two_bytes = &data[data.len() - u16_bytes..];
        let num_entry = u16::from_le_bytes(last_two_bytes.try_into().unwrap()) as usize;

        // 计算并检查总的元数据大小
        let metadata_size = num_entry * u16_bytes + u16_bytes;
        if data.len() < metadata_size {
            panic!("Invalid data: too short to contain offset array");
        }

        // 获取偏移量数组的字节切片
        let offsets_slice_u8 = &data[data.len() - metadata_size..data.len() - u16_bytes];

        // 安全地将字节转换为 u16 数组
        let offsets: Vec<u16> = offsets_slice_u8
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        // 获取实际数据
        let data_slice = &data[..data.len() - metadata_size];

        Block {
            data: data_slice.to_vec(),
            offsets,
        }
    }

    pub fn get_first_key(&self) -> KeyVec {
        let mut buf = &self.data[..];
        let key_len = buf.get_u16();
        let key = &buf[..key_len as usize];
        KeyVec::from_vec(key.to_vec())
    }
}
