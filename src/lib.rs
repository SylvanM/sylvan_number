#![feature(bigint_helper_methods)]

pub mod big_number;
pub mod int_utility;

#[cfg(test)]
mod tests {

    // MARK: Utility tests

    use algebra_kit::algebra::{EuclideanDomain, Ring};
    use rand::{thread_rng, Rng};

    use crate::big_number::UBigNumber;

    #[test]
    fn test_conversions() {
        let ubn: UBigNumber = "0x000000010000000200000003000000040000000500000006".into();
        assert_eq!(ubn.words, vec![0x0000000500000006, 0x0000000300000004, 0x0000000100000002]);
    }

    #[test]
    fn test_printing() {
        let ubn = UBigNumber::from_words(vec![1, 2, 9, 8, 4, 1]);
        println!("{:?}", ubn);
    }

    // MARK: Arithmetic Tests

    #[test]
    fn test_add() {
        let a: UBigNumber = "0x000000010000000200000003000000040000000500000006".into();
        let b: UBigNumber = "0x000000010000000100000001000000010000000100000001".into();
        assert_eq!(a + b, "0x000000020000000300000004000000050000000600000007".into());

        let c: UBigNumber = "0xFFFFFFFFFFFFFFFF".into();
        assert_eq!(c + 1.into(), "0x10000000000000000".into());

        // I wonder what happens here
        println!("{:?}", UBigNumber::from_int(1) - 2.into());


    }

    #[test]
    fn test_sub() {
        let a: UBigNumber = "0x103C57F8C7B4F5651".into();
        let b = UBigNumber::one();

        println!("{:?}", a - b);
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            UBigNumber::from_hex_string("0x558CE7C54D02B1FC4F41C55BD511D549D8A6C8F64F06BCAAB23FF1DE295198E9") *
            UBigNumber::from_hex_string("0x9779F079B986C0AB28067950DB40BB87AA1FCA6C89DA76AA689A47918E060C78"),
            "0x329edcabb3905d57a66426b26955d284a9020c8907621ae56070df059c7658ae6900b41a962383d7facf2bb89c11c94deffcfa00ddafa441203a3fc704e09938".into()
        )
    }

    #[test]
    fn test_clone() {
        let a = UBigNumber::rand(4);

        let _ = a.clone() + a;
    }

    #[test]
    fn test_short_division() {
        
        // Some randomly generated, known cases.

        {
            let dividend: UBigNumber = "0xCDF6CB3091A77FE6143FC6910875333BB3B08D7AE0B60629".into();
            let divisor: UBigNumber = "0xD4024CBD9C6BAE10".into();
            let correct_quotient: UBigNumber = "f8b36482cdc11eae615a3c1c9c8ad986".into();
            let correct_remainder: UBigNumber = "43b4eec6252d59c9".into();
            assert_eq!(correct_quotient, dividend.clone() / divisor.clone());
            assert_eq!(correct_remainder, dividend % divisor);
        }

        for _ in 0..100 {
            let divisor = UBigNumber::from_int(rand::thread_rng().gen());
            let dividend = UBigNumber::rand(rand::thread_rng().gen_range(1..10));
            let (q, r) = dividend.quotient_and_remainder(&divisor);
            assert_eq!(divisor * q + r, dividend);
        }

    }

    #[test]
    fn test_division() {

        // A known case, where we are computing (2 * rand) / rand,
        // so we should get two.
        {
            let known_rand: UBigNumber = "0x56C1ADE683B78C807948E66BDA765CC9BA2FB6F85667311E".into();
            let twice = known_rand.clone() * 2.into();
            let (q, r) = twice.quotient_and_remainder(&known_rand);
            assert_eq!(q, 2.into());
            assert_eq!(r, UBigNumber::zero());
        }

        assert_eq!(UBigNumber::from_int(1) / 1.into(), 1.into());
        for _ in 0..10 {
            let rand = UBigNumber::rand(3); // arbitrary size
            assert_eq!(rand.clone() / 1.into(), rand);
            assert_eq!(rand.clone() / rand.clone(), 1.into());

            let twice = rand.clone() * 2.into();
            let quotient = twice / rand;

            assert_eq!(quotient, 2.into());

        }

        {
            let dividend: UBigNumber = "0x55589105C1E8687FEDA2729CA4FBD7DF".into();
            let divisor: UBigNumber = "0x21906BFD894BDCD7F0F5A4CC17554F5F".into();
            let correct_quotient: UBigNumber = 2.into();
            let correct_remainder: UBigNumber = "1237b90aaf50aed00bb7290476513921".into();
            assert_eq!(correct_quotient, dividend.clone() / divisor.clone());
            assert_eq!(correct_remainder, dividend % divisor);
        }

        {
            let dividend: UBigNumber = "0x7FEB1182A1B069E520AD55537A7E6FF76A5E0258CF105762".into();
            let divisor: UBigNumber = "0x63861802BBE83994FAA714D6517E1784".into();
            let correct_quotient: UBigNumber = "149099cbfc9ba7b38".into();
            let correct_remainder: UBigNumber = "24919ff5ff5ce6f9ca0f48bfac46c682".into();
            assert_eq!(correct_quotient, dividend.clone() / divisor.clone());
            assert_eq!(correct_remainder, dividend % divisor);
        }
        
        // Just realized I can have just a bare scope on its own! Cool! Now I will be using this so I can reuse the same 
        // variable names.
        {
            let dividend: UBigNumber = "0x8257DC4F1B654A9C47B4FD75965CC3AD59B5F00C06AE76C0B322CBD40CA7391E583158A3F4EA59631C1099FA4D7AFACDF481AA5CF4F3AF4A91B859F25FE8F1F4A8E6865CF228831FAC53FCE908880E5".into();
            let divisor: UBigNumber = "0xEC3857AB7272481CFC9E4B7A828EFB861B005130E5F2F301".into();
            let correct_quotient: UBigNumber = "8d41ebf26d8e036e2180bfa06594cea61ab6cba1834e249f3d06a9d3f2e07700ac2a7984db2956c2267ab39b9656d3e68338034da2edc0e".into();
            let correct_remainder: UBigNumber = "9e1e5b3c0fd14f30866c9465fc8dae310053b8bab03c5ad7".into();
            assert_eq!(correct_quotient, dividend.clone() / divisor.clone());
            assert_eq!(correct_remainder, dividend % divisor);
        }

        {
            let dividend = UBigNumber::from_words(vec![1, 2, 9, 8, 4, 1]);
            let divisor = UBigNumber::from_words(vec![2, 4, 8]);
            let (q, r) = dividend.quotient_and_remainder(&divisor);
            assert_eq!(q * divisor + r, dividend);
        }

        for _ in 0..100 {
            let dividend = UBigNumber::rand(rand::thread_rng().gen_range(4..10));
            let divisor = UBigNumber::rand(rand::thread_rng().gen_range(1..dividend.len()));
            let (q, r) = dividend.quotient_and_remainder(&divisor);
            assert_eq!(dividend, divisor * q + r);
        }

    }

}
