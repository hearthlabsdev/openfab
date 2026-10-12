use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileFormat {
    GCode,
    STL,
    /// 3MF with specification for which vendor parsing to use.
    ThreeMF(String),
    OBJ,
    PNG,
    SVG,
}
