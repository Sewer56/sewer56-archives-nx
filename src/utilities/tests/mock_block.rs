use super::packer_file_for_testing::PackerFileForTesting;
use crate::{
    api::traits::*,
    implementation::pack::blocks::polyfills::{Block, PtrEntry},
};
use alloc::rc::Rc;
use core::any::Any;
use hashbrown::HashTable;

// Mock implementation of required traits for testing
pub struct MockBlock {
    dict_index: u32,
}

impl<T> Block<T> for MockBlock
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn append_items(&self, _: &mut Vec<Rc<T>>, _: &mut HashTable<PtrEntry>) {}
    fn items(&self) -> &[Rc<T>] {
        &[]
    }
    fn max_decompressed_block_offset(&self) -> u32 {
        0
    }
}

impl HasDictIndex for MockBlock {
    fn dict_index(&self) -> u32 {
        self.dict_index
    }
}

pub fn create_mock_block(dict_index: u32) -> Box<dyn Block<PackerFileForTesting>> {
    Box::new(MockBlock { dict_index })
}
