use crate::{modified_scrape::errors::PVSSError, Scalar};

use ark_ec::PairingEngine;
use ark_ff::Zero;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read, SerializationError, Write};


/* Struct PVSSShare models the "core" of a PVSS sharing generated by the a participant when acting as dealer */

#[derive(CanonicalSerialize, CanonicalDeserialize, Clone, Debug, PartialEq)]
pub struct PVSSCore<E>
where
    E: PairingEngine,
{
    pub encs: Vec<E::G1Projective>,   // vector of encryptions c
    pub comms: Vec<E::G2Projective>,  // vector of commitments v
}

impl<E> PVSSCore<E>
where
    E: PairingEngine,
{
    // Create a new "empty" PVSS core, where all fields are set to "zero" values.
    pub fn empty(num_participants: usize) -> Self {
        PVSSCore {
	        encs:  vec![E::G1Projective::zero(); num_participants],
	        comms: vec![E::G2Projective::zero(); num_participants],
        }
    }


    // Aggregation of two PVSSCore instances.
    pub fn aggregate(&self, other: &Self) -> Result<Self, PVSSError<E>> {
        // Perform some basic checks:

        // Commitment vector must be non-empty
        if self.comms.len() == 0 {
            return Err(PVSSError::EmptyEncryptionsVectorError);
        }

        // Commitment vector lengths must match
        if self.comms.len() != other.comms.len() {
            return Err(PVSSError::MismatchedCommitmentsError(self.comms.len(), other.comms.len()));
        }

        // Encryption vector lengths must match
        if self.encs.len() != other.encs.len() {
            return Err(PVSSError::MismatchedEncryptionsError(self.encs.len(), other.encs.len()));
        }

        // Commitment and encryption vector lengths must match
        if self.comms.len() != self.encs.len() {
            return Err(PVSSError::MismatchedCommitmentsEncryptionsError(self.comms.len(), other.encs.len()));
        }

        // Aggregate PVSS cores
        let result = Self {
                encs: self
                    .encs
                    .iter()
                    .zip(other.encs.iter())
                    .map(|(e1, e2)| *e1 + *e2)
                    .collect::<Vec<_>>(),
                comms: self
                    .comms
                    .iter()
                    .zip(other.comms.iter())
                    .map(|(c1, c2)| *c1 + *c2)
                    .collect::<Vec<_>>(),
        };

        Ok(result)
    }

}


// PVSSShareSecrets models the secret parts underlying each share.
pub struct PVSSShareSecrets<E: PairingEngine> {
    pub p_0: Scalar<E>,           // secret polynomial's free term s s.t.: p_i(0) = s
    pub my_secret: E::G1Affine,   // partial secret
}


/* Unit tests: */

#[cfg(test)]
mod test {

    use crate::signature::utils::tests::check_serialization;

    use std::ops::Neg;

    use super::PVSSCore;

    use ark_ff::Zero;
    use ark_ec::PairingEngine;
    use ark_std::UniformRand;
    use ark_bls12_381::{
	    Bls12_381 as E,   // type Bls12_381 = Bls12<Parameters> (Bls12 implements PairingEngine)
    };
    use rand::thread_rng;

    #[test]
    fn test_empty() {
        let size: usize = 10;

	    let core = PVSSCore::<E>::empty(size);

	    assert!(core.encs.iter().all(|&x| x == <E as PairingEngine>::G1Projective::zero()));
	    assert!(core.comms.iter().all(|&x| x == <E as PairingEngine>::G2Projective::zero()));
    }

    #[test]
    fn test_aggregate() {
        let rng = &mut thread_rng();
        let size: usize = 10;

        let encs = vec![<E as PairingEngine>::G1Projective::rand(rng); size];
        let comms = vec![<E as PairingEngine>::G2Projective::rand(rng); size];

	    let core1 = PVSSCore::<E> {
            encs: encs.clone(),
            comms: comms.clone(),
        };

        let core2 = PVSSCore::<E> {
            encs: encs.iter().map(|&x| x.neg()).collect(),
            comms: comms.iter().map(|&x| x.neg()).collect(),
        };

        let result = core1.aggregate(&core2).unwrap();

        assert!(result.encs.iter().all(|&x| x == <E as PairingEngine>::G1Projective::zero()));
        assert!(result.comms.iter().all(|&x| x == <E as PairingEngine>::G2Projective::zero()));
    }

    #[test]
    #[should_panic]
    fn test_aggregate_empty_encs() {
        let size = 10;

        let core1 = PVSSCore::<E> {
	        encs:  vec![],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size],
        };

        let core2 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size],
        };

        core1.aggregate(&core2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_aggregate_empty_comms() {
        let size = 10;

        let core1 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size],
	        comms: vec![],
        };

        let core2 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size],
        };

        core1.aggregate(&core2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_aggregate_mismatching_encs_lengths() {
        let size1 = 10;
        let size2 = 20;

        let core1 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size1],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size1],
        };

        let core2 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size2],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size2],
        };

        core1.aggregate(&core2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_aggregate_mismatching_encs_comms_lengths() {
        let size = 10;

        let core1 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size+1],
        };

        let core2 = PVSSCore::<E> {
	        encs:  vec![<E as PairingEngine>::G1Projective::zero(); size],
	        comms: vec![<E as PairingEngine>::G2Projective::zero(); size],
        };

        core1.aggregate(&core2).unwrap();
    }

    #[test]
    fn test_serialization() {
        let rng = &mut thread_rng();
        let size = 10;

	    let core = PVSSCore::<E> {
            encs: vec![<E as PairingEngine>::G1Projective::rand(rng); size],
            comms: vec![<E as PairingEngine>::G2Projective::rand(rng); size],
        };

        check_serialization(core.clone());
    }

}
