use std::convert::TryFrom;

#[cfg(test)]
mod tests {
    use c_defines_to_enum::parse_c_defines_to_enum;
    use std::convert::TryFrom;


    parse_c_defines_to_enum!(
            TestEnum,
            remove_prefix = "SYS",
            to_lower = true,
            content = include_str!("test.h")
        );

    #[test]
    fn it_works() {
        println!("{:?}", TestEnum::try_from(1234));
        let value: usize = TestEnum::test1.into();
        println!("{:?} = {}", TestEnum::test1, value);
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
enum Test2 {
    c1 = 11,
    c2 = 22,
}

impl TryFrom<usize> for Test2 {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            11 => Ok(Self::c1),
            22 => Ok(Self::c2),
            _ => Err(())
        }
    }
}

impl Into<usize> for Test2 {
    fn into(self) -> usize {
        self as usize
    }
}
