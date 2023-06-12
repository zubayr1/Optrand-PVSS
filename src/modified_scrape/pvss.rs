use ark_ec::PairingEngine;
use ark_ff::Zero;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read, SerializationError, Write};

use crate::signature::schnorr::{SchnorrSignature};   // replace with our final choice of signature scheme
use crate::nizk::{dlk::{DLKProof, srs::SRS as DLKSRS}, scheme::NIZKProof};

use super::{decomp::DecompProof, config::Config};
use crate::modified_scrape::errors::PVSSError;
use crate::signature::scheme::BatchVerifiableSignatureScheme;
use crate::modified_scrape::poly::Scalar;
use crate::modified_scrape::decomp::ProofGroup;

use std::marker::PhantomData;


/* Struct PVSSShare models the PVSS sharing generated by the a participant when acting as dealer */

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct PVSSShare<E, SSIG>
where
    E: PairingEngine,
    SSIG: BatchVerifiableSignatureScheme<PublicKey = E::G1Affine, Secret = Scalar<E>>   // Double-check PublicKey (drop constraint if unnecessary)
{
    pub comms: Vec<E::G2Projective>,    	     // vector of commitments v
    pub encs: Vec<E::G1Projective>,     	     // vector of encryptions c
    pub decomp_proof: Option<DecompProof<E>>,        // decomposition proof (contains gs)
    pub sig_of_knowledge: Option<SSIG::Signature>,   // adjust type according to our choice of signature scheme
}

impl<E, SSIG> PVSSShare<E, SSIG>
where
    E: PairingEngine,
    SSIG: BatchVerifiableSignatureScheme<PublicKey = E::G1Affine, Secret = Scalar<E>>   // Double-check PublicKey (drop constraint if unnecessary)
{

    // Sharing API

    // Create a new "empty" PVSSShare, where all fields are set to "zero" values
    pub fn empty(config: &Config<E>) -> Self {
        PVSSShare {
	    comms: vec![E::G2Projective::zero(); config.num_replicas],
	    encs: vec![E::G1Projective::zero(); config.num_replicas],
	    decomp_proof: None,
            sig_of_knowledge: None
        }
    }



/*
    // Aggregation of PVSSShare instances
    pub fn aggregate(&self, other: &Self) -> Self {
	

	Self {  }
    }
*/



}


// PVSSAggregatedShare models an aggregation of individual PVSSShare instances.
#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct PVSSAggregatedShare<E, SSIG>
where
    E: PairingEngine,
    SSIG: BatchVerifiableSignatureScheme<PublicKey = E::G1Affine, Secret = Scalar<E>>   // Double-check PublicKey (drop constraint if unnecessary)
{
    pub comms: Vec<E::G2Projective>,    	                             // vector of commitments v
    pub encs: Vec<E::G1Projective>,     	                             // vector of encryptions c
    pub decomp_proofs: Vec<<DLKProof<ProofGroup<E>> as NIZKProof>::Proof>,   // vector of decomposition proofs
    pub id_vec: Vec<usize>, 			                             // vector of participant ids that submitted PVSS shares
    pub sig_scheme: std::marker::PhantomData<SSIG>                           // for caching SSIG
}


impl<E, SSIG> PVSSAggregatedShare<E, SSIG> 
where
    E: PairingEngine,
    SSIG: BatchVerifiableSignatureScheme<PublicKey = E::G1Affine, Secret = Scalar<E>>   // Double-check PublicKey (drop constraint if unnecessary)
{

    // Associated method for aggregating the contents of a vector of PVSS shares into a
    // single aggregated PVSS share, also containing the participant identities.
    pub fn aggregate(shares: &Vec<PVSSShare<E, SSIG>>, ids: Vec<usize>) -> Result<PVSSAggregatedShare<E, SSIG>, PVSSError<E>> {
	if shares.len() == 0 {
	    return Err(PVSSError::<E>::EmptySharesVectorError);
	}

	if shares.len() > ids.len() {
	    return Err(PVSSError::<E>::InsufficientIdsError);
	}

	let n = shares[0].encs.len();
	let to_agg = shares.len();

	let mut encs = vec![E::G1Projective::zero(); n];
	let mut comms = vec![E::G2Projective::zero(); n];
	let mut decomp_proofs = vec![];

	for i in 0..to_agg {
	    let share = &shares[i];

	    // Basic checks
	    if share.comms.len() != n {
		return Err(PVSSError::<E>::InsufficientCommitsInShareError(share.comms.len(), n));
	    }

	    if share.encs.len() != n {
		return Err(PVSSError::<E>::InsufficientEncryptionsInShareError(share.encs.len(), n));
	    }

	    // Aggregate commitments and encryptions
	    for j in 0..n {
		// Combine commitments
		comms[j] += share.comms[j];

		// Combine encryptions
		encs[j] += share.encs[j];
	    }

	    // Aggregate decomposition proofs
	    decomp_proofs.push(share.decomp_proof.clone().unwrap().proof);
	}

	Ok(PVSSAggregatedShare { comms, encs, decomp_proofs, id_vec: ids, sig_scheme: PhantomData })
    }
}


/*
// PVSSShareSecrets models the secret parts underlying each share.
pub struct PVSSShareSecrets<E: PairingEngine> {
    pub p_0: Scalar<E>,           // secret s s.t.: p_i(0) = s
    pub my_secret: E::G2Affine,   // actual secret; is this one correct???
}
*/
