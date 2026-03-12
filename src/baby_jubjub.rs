//! EIP-2494 Baby Jubjub Curve
//!
//! This is an append to the the `ark-ed-on-bn254` crate to use the EIP-2494 defined Baby Jubjub curve parameters.
//!
//! - <https://eips.ethereum.org/EIPS/eip-2494>
//!
//! - Base field: q = 21888242871839275222246405745257275088548364400416034343698204186575808495617
//! - Scalar field: r = 2736030358979909402780800718157159386076813972158567259200215660948447373041
//! - Order: n = l * cofactor = 21888242871839275222246405745257275088614511777268538073601725287587578984328
//! - Cofactor: 8
//! - Subgroup order: l = 2736030358979909402780800718157159386076813972158567259200215660948447373041
//! - Curve equation: ax² + y² = 1 + d·x²y², where
//!    - a = 168700
//!    - d = 168696
//! - Generator point:
//!   (995203441582195749578291179787384436505546430278305826713579947235728471134,
//!   5472060717959818805561601436314318772137091100104008585924551046643952123905)
//! - Base point:
//!   (5299619240641551281634865583518297030282874472190772894086521144482721001553,
//!   16950150798460657717958625567821834550301663161624707787222815936182638968203)

use ark_ec::{
    models::CurveConfig,
    twisted_edwards::{Affine, MontCurveConfig, Projective, TECurveConfig},
};
use ark_ed_on_bn254::{Fq, Fr};
use ark_ff::{BigInt, Field};

pub type EdwardsAffine = Affine<BabyJubjubConfig>;
pub type EdwardsProjective = Projective<BabyJubjubConfig>;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct BabyJubjubConfig;

impl CurveConfig for BabyJubjubConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    // h = 8
    const COFACTOR: &'static [u64] = &[8];

    // h^(-1) (mod r)
    const COFACTOR_INV: Fr = Fr::new(BigInt([
        1910727873761444371,
        3879587201322635337,
        10387772030373733802,
        381390435431574916,
    ]));
}

// Twisted Edwards form
// ax^2 + y^2 = 1 + dx^2y^2
impl TECurveConfig for BabyJubjubConfig {
    // a = 168700
    const COEFF_A: Fq = Fq::new(BigInt([168700, 0, 0, 0]));

    #[inline(always)]
    fn mul_by_a(elem: Self::BaseField) -> Self::BaseField {
        elem * <BabyJubjubConfig as TECurveConfig>::COEFF_A
    }

    // d = 168696
    const COEFF_D: Fq = Fq::new(BigInt([168696, 0, 0, 0]));

    // Base point is used as generator to operate in subgroup
    const GENERATOR: EdwardsAffine = EdwardsAffine::new_unchecked(BASE_X, BASE_Y);

    type MontCurveConfig = BabyJubjubConfig;
}

// Montgomery form
// By^2 = x^3 + A x^2 + x
impl MontCurveConfig for BabyJubjubConfig {
    // A = 168698
    const COEFF_A: Fq = Fq::new(BigInt([168698, 0, 0, 0]));
    // B = 1
    const COEFF_B: Fq = Fq::ONE;

    type TECurveConfig = BabyJubjubConfig;
}

/// Generator point x-coordinate
pub const GENERATOR_X: Fq = Fq::new(BigInt([
    4680394886406779998,
    13012219997379977915,
    4088347315492242861,
    158545055271577405,
]));
/// Generator point y-coordinate
pub const GENERATOR_Y: Fq = Fq::new(BigInt([
    5834551189936537601,
    5335914614254099492,
    7931984006246061591,
    871749566700742666,
]));

/// Subgroup order `l`
pub const SUBGROUP_ORDER: BigInt<4> = BigInt([
    7454187305358665457,
    12339561404529962506,
    3965992003123030795,
    435874783350371333,
]);

// Subgroup generator
// Generates subgroup l * P = O

/// Base point x-coordinate
pub const BASE_X: Fq = Fq::new(BigInt([
    2923948824128221265,
    3078447844201652406,
    5669102708735506369,
    844278054434796443,
]));
/// Base point y-coordinate
pub const BASE_Y: Fq = Fq::new(BigInt([
    5421249259631377803,
    18221569726161695607,
    2690670003684637165,
    2700314812950295113,
]));

#[cfg(test)]
mod tests {
    //! Implementation of the tests presented in the EIP-2494

    use super::*;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ff::{PrimeField, Zero};

    fn fq_from_limbs(limbs: [u64; 4]) -> Fq {
        Fq::new(BigInt(limbs))
    }

    #[test]
    fn test_addition() {
        let p1 = EdwardsAffine::new_unchecked(
            fq_from_limbs([
                7699839232871213100,
                11377594457309997372,
                11223204053576433119,
                2832127448816122262,
            ]),
            fq_from_limbs([
                5014193860014684243,
                12891415696119316661,
                16351138293120604662,
                418439792016356479,
            ]),
        );

        let p2 = EdwardsAffine::new_unchecked(
            fq_from_limbs([
                8787143617238384983,
                14452031502512802791,
                7941460534247489132,
                2635075998581887399,
            ]),
            fq_from_limbs([
                3786921421244027607,
                17990144301093405137,
                1791269356123149699,
                3316665278388780408,
            ]),
        );

        let result = (p1 + p2).into_affine();

        assert_eq!(
            result,
            EdwardsAffine::new_unchecked(
                fq_from_limbs([
                    10405464339497284449,
                    8277032624866824417,
                    12950120428123253884,
                    1261101424013096072,
                ]),
                fq_from_limbs([
                    11601515926616747027,
                    759323840248000415,
                    18014248375850445194,
                    2235942773966081583,
                ])
            )
        );
    }

    #[test]
    fn test_doubling() {
        let p1 = EdwardsAffine::new_unchecked(
            fq_from_limbs([
                7699839232871213100,
                11377594457309997372,
                11223204053576433119,
                2832127448816122262,
            ]),
            fq_from_limbs([
                5014193860014684243,
                12891415696119316661,
                16351138293120604662,
                418439792016356479,
            ]),
        );

        let result = (p1 + p1).into_affine();

        assert_eq!(
            result,
            EdwardsAffine::new_unchecked(
                fq_from_limbs([
                    6573998244250000053,
                    15447716325968879609,
                    5178469352008426472,
                    1097776659210999491,
                ]),
                fq_from_limbs([
                    5726854610579821793,
                    16363202296762874374,
                    6162349764539256418,
                    691182090570128499,
                ])
            )
        );
    }

    #[test]
    fn test_doubling_identity() {
        let identity = EdwardsAffine::new_unchecked(Fq::zero(), Fq::ONE);
        let result = (identity + identity).into_affine();

        assert_eq!(result, identity);
    }

    #[test]
    fn test_curve_membership() {
        let valid_point = EdwardsAffine::new_unchecked(Fq::zero(), Fq::ONE);
        assert!(valid_point.is_on_curve());

        let invalid_point = EdwardsAffine::new_unchecked(Fq::ONE, Fq::zero());
        assert!(!invalid_point.is_on_curve());
    }

    #[test]
    fn test_base_point_choice() {
        let g = EdwardsAffine::new_unchecked(GENERATOR_X, GENERATOR_Y);

        let expected_base_point = EdwardsAffine::new_unchecked(BASE_X, BASE_Y);
        let cofactor = Fr::from_be_bytes_mod_order(&[BabyJubjubConfig::COFACTOR[0] as u8]);
        let calculated_base_point = (g * cofactor).into_affine();

        assert_eq!(calculated_base_point, expected_base_point);
    }

    #[test]
    fn test_base_point_order() {
        let base_point = EdwardsAffine::new_unchecked(BASE_X, BASE_Y);

        let result = base_point.mul_bigint(SUBGROUP_ORDER).into_affine();
        let identity = EdwardsAffine::new_unchecked(Fq::zero(), Fq::ONE);

        assert_eq!(result, identity);
    }
}
