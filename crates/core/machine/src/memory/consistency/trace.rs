use p3_field::PrimeField32;

use bf_core_executor::events::{
    ByteRecord, MemoryReadRecord, MemoryRecord, MemoryRecordEnum, MemoryWriteRecord,
};

use super::{MemoryAccessCols, MemoryReadCols, MemoryReadWriteCols, MemoryWriteCols};

impl<F: PrimeField32> MemoryWriteCols<F> {
    pub fn populate(&mut self, record: MemoryWriteRecord, output: &mut impl ByteRecord) {
        let current_record = MemoryRecord { value: record.value, timestamp: record.timestamp };
        let prev_record =
            MemoryRecord { value: record.prev_value, timestamp: record.prev_timestamp };
        self.prev_value = F::from_canonical_u8(prev_record.value);
        self.access.populate_access(current_record, prev_record, output);
    }
}

impl<F: PrimeField32> MemoryReadCols<F> {
    pub fn populate(&mut self, record: MemoryReadRecord, output: &mut impl ByteRecord) {
        let current_record = MemoryRecord { value: record.value, timestamp: record.timestamp };
        let prev_record = MemoryRecord { value: record.value, timestamp: record.prev_timestamp };
        self.access.populate_access(current_record, prev_record, output);
    }
}

impl<F: PrimeField32> MemoryReadWriteCols<F> {
    pub fn populate(&mut self, record: MemoryRecordEnum, output: &mut impl ByteRecord) {
        match record {
            MemoryRecordEnum::Read(read_record) => self.populate_read(read_record, output),
            MemoryRecordEnum::Write(write_record) => self.populate_write(write_record, output),
        }
    }

    pub fn populate_write(&mut self, record: MemoryWriteRecord, output: &mut impl ByteRecord) {
        let current_record = MemoryRecord { value: record.value, timestamp: record.timestamp };
        let prev_record =
            MemoryRecord { value: record.prev_value, timestamp: record.prev_timestamp };
        self.prev_value = F::from_canonical_u8(prev_record.value);
        self.access.populate_access(current_record, prev_record, output);
    }

    pub fn populate_read(&mut self, record: MemoryReadRecord, output: &mut impl ByteRecord) {
        let current_record = MemoryRecord { value: record.value, timestamp: record.timestamp };
        let prev_record = MemoryRecord { value: record.value, timestamp: record.prev_timestamp };
        self.prev_value = F::from_canonical_u8(prev_record.value);
        self.access.populate_access(current_record, prev_record, output);
    }
}

impl<F: PrimeField32> MemoryAccessCols<F> {
    pub(crate) fn populate_access(
        &mut self,
        current_record: MemoryRecord,
        prev_record: MemoryRecord,
        _output: &mut impl ByteRecord,
    ) {
        self.value = F::from_canonical_u8(current_record.value);

        self.prev_clk = F::from_canonical_u32(prev_record.timestamp);

        let prev_time_value = prev_record.timestamp;
        let current_time_value = current_record.timestamp;

        let diff_minus_one = current_time_value - prev_time_value - 1;
        let diff_16bit_limb = (diff_minus_one & 0xffff) as u16;
        self.diff_16bit_limb = F::from_canonical_u16(diff_16bit_limb);
        let diff_8bit_limb = (diff_minus_one >> 16) & 0xff;
        self.diff_8bit_limb = F::from_canonical_u32(diff_8bit_limb);

        // Add a byte table lookup with the 16Range op.
        // output.add_u16_range_check(diff_16bit_limb);

        // Add a byte table lookup with the U8Range op.
        // output.add_u8_range_check(diff_8bit_limb as u8);
    }
}
