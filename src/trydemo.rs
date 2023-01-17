#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate bellman;
extern crate pairing;
extern crate rand;

use std::num::NonZeroI128;

// For randomness (during paramgen and proof generation)
use self::rand::{thread_rng, Rng};

// Bring in some tools for using pairing-friendly curves
use self::pairing::{
    Engine,
    Field,
    PrimeField
};

// We're going to use the BLS12-381 pairing-friendly elliptic curve.
use self::pairing::bls12_381::{
    Bls12,
    Fr
};

// We'll use these interfaces to construct our circuit.
use self::bellman::{
    Circuit,
    ConstraintSystem,
    SynthesisError
};

// We're going to use the Groth16 proving system.
use self::bellman::groth16::{
    Proof,
    generate_random_parameters,
    prepare_verifying_key,
    create_random_proof,
    verify_proof,
};

//证明我知道方程式：x^2+y^2=z^2的一组解。
pub struct CubeDemo<E: Engine> {
    pub x: Option<E::Fr>,
    pub y: Option<E::Fr>,
    pub z: Option<E::Fr>,
}

impl <E: Engine> Circuit<E> for CubeDemo<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self, 
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {
        //拍平：
        //x * x = x_square
        //y * y = y_square
        //z * z = z_square
        //(x_square+y_square) * 1 = z_square

        // Allocate the first private "auxiliary" variable
        let x_val = self.x;
        let x = cs.alloc(|| "x", || {
            x_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        //分配x * x = x_square
        let x_square_val = x_val.map(|mut e| {
            e.square();
            e
        });
        let x_square = cs.alloc(|| "x_square", || {
            x_square_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Enforce: x * x = x_square
        cs.enforce(
            || "x_square",
            |lc| lc + x,
            |lc| lc + x,
            |lc| lc + x_square
        );

        //分配第二个隐私输入变量y
        let y_val = self.y;
        let y = cs.alloc(|| "y", || {
            y_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        //分配y * y = y_square
        let y_square_val = y_val.map(|mut e| {
            e.square();
            e
        });
        let y_square = cs.alloc(|| "y_square", || {
            y_square_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        //Enforce: y * y = y_square
        cs.enforce(
            || "y_square",
            |lc| lc + y,
            |lc| lc + y,
            |lc| lc + y_square
        );

        //分配第三个隐私输入变量z
        let z_val = self.z;
        let z = cs.alloc(|| "z", || {
            z_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        //分配z * z = z_square
        let z_square_val = z_val.map(|mut e| {
            e.square();
            e
        });
        let z_square = cs.alloc(|| "z_square", || {
            z_square_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        //Enforce: z * z = z_square
        cs.enforce(
            || "z_square",
            |lc| lc + z,
            |lc| lc + z,
            |lc| lc + z_square
        );
        
        //分配一个公共输出
        let out = cs.alloc_input(|| "out", || {
            let mut tmp = x_square_val.unwrap();
            tmp.add_assign(&y_square_val.unwrap());
            tmp.sub_assign(&z_square_val.unwrap());
            Ok(tmp)
        })?; 

        //(x_square + y_square-z_square) * 1 = 0
        cs.enforce(
            || "trydemo",
            |lc| lc + x_square + y_square-z_square,
            |lc| lc + CS::one(),
            |lc| lc + out
        );

        Ok(())

    }
}


#[test]
fn test_trydemo_proof(){
    let rng = &mut thread_rng();

    println!("Creating parameters...");
    let params ={
        let c = CubeDemo::<Bls12>{
            x: None,
            y: None,
            z: None,
        };

        generate_random_parameters(c,rng).unwrap()

    };

    let pvk = prepare_verifying_key(&params.vk);

    println!("Creating proofs...");

   

    let c = CubeDemo::<Bls12>{
        x: Fr::from_str("3"),
        y: Fr::from_str("4"),
        z: Fr::from_str("5")
    };

    let proof = create_random_proof(c,&params,rng).unwrap();


   

    assert!(verify_proof(
        &pvk,
        &proof,
        &[Fr::from_str("0").unwrap()]
    ).unwrap());

}
