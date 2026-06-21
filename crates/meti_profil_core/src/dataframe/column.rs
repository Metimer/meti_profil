use arrow::array::ArrayRef;
use arrow::datatypes::DataType;

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub array: ArrayRef,
}

impl Column {
    pub fn new(name: impl Into<String>, array: ArrayRef) -> Self {
        let data_type = array.data_type().clone();
        Self {
            name: name.into(),
            data_type,
            array,
        }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }
}
