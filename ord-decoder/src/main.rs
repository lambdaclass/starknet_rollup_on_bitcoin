use std::{collections::BTreeMap, env, iter::Peekable, println};

use bitcoin::{
    blockdata::{
        opcodes,
        script::{self, Instruction, Instructions},
    },
    util::taproot::TAPROOT_ANNEX_PREFIX,
    Script, Witness,
};
const PROTOCOL_ID: &[u8] = b"ord";

const BODY_TAG: &[u8] = &[];
const CONTENT_TYPE_TAG: &[u8] = &[1];

#[derive(Debug, PartialEq)]
enum InscriptionError {
    EmptyWitness,
    InvalidInscription,
    KeyPathSpend,
    NoInscription,
    Script(script::Error),
    UnrecognizedEvenField,
}

type Result<T, E = InscriptionError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Inscription {
    body: Option<Vec<u8>>,
    content_type: Option<Vec<u8>>,
}

struct InscriptionParser<'a> {
    instructions: Peekable<Instructions<'a>>,
}

impl<'a> InscriptionParser<'a> {
    fn parse(witness: &Witness) -> Result<Inscription> {
        if witness.is_empty() {
            return Err(InscriptionError::EmptyWitness);
        }

        if witness.len() == 1 {
            return Err(InscriptionError::KeyPathSpend);
        }

        let annex = witness
            .last()
            .and_then(|element| element.first().map(|byte| *byte == TAPROOT_ANNEX_PREFIX))
            .unwrap_or(false);

        if witness.len() == 2 && annex {
            return Err(InscriptionError::KeyPathSpend);
        }

        let script = witness
            .iter()
            .nth(if annex {
                witness.len() - 1
            } else {
                witness.len() - 2
            })
            .unwrap();

        InscriptionParser {
            instructions: Script::from(Vec::from(script)).instructions().peekable(),
        }
        .parse_script()
    }

    fn parse_script(mut self) -> Result<Inscription> {
        loop {
            let next = self.advance()?;

            if next == Instruction::PushBytes(&[]) {
                if let Some(inscription) = self.parse_inscription()? {
                    return Ok(inscription);
                }
            }
        }
    }

    fn advance(&mut self) -> Result<Instruction<'a>> {
        self.instructions
            .next()
            .ok_or(InscriptionError::NoInscription)?
            .map_err(InscriptionError::Script)
    }

    fn parse_inscription(&mut self) -> Result<Option<Inscription>> {
        if self.advance()? == Instruction::Op(opcodes::all::OP_IF) {
            if !self.accept(Instruction::PushBytes(PROTOCOL_ID))? {
                return Err(InscriptionError::NoInscription);
            }

            let mut fields = BTreeMap::new();

            loop {
                match self.advance()? {
                    Instruction::PushBytes(BODY_TAG) => {
                        let mut body = Vec::new();
                        while !self.accept(Instruction::Op(opcodes::all::OP_ENDIF))? {
                            body.extend_from_slice(self.expect_push()?);
                        }
                        fields.insert(BODY_TAG, body);
                        break;
                    }
                    Instruction::PushBytes(tag) => {
                        if fields.contains_key(tag) {
                            return Err(InscriptionError::InvalidInscription);
                        }
                        fields.insert(tag, self.expect_push()?.to_vec());
                    }
                    Instruction::Op(opcodes::all::OP_ENDIF) => break,
                    _ => return Err(InscriptionError::InvalidInscription),
                }
            }

            let body = fields.remove(BODY_TAG);
            let content_type = fields.remove(CONTENT_TYPE_TAG);

            for tag in fields.keys() {
                if let Some(lsb) = tag.first() {
                    if lsb % 2 == 0 {
                        return Err(InscriptionError::UnrecognizedEvenField);
                    }
                }
            }

            return Ok(Some(Inscription { body, content_type }));
        }

        Ok(None)
    }

    fn expect_push(&mut self) -> Result<&'a [u8]> {
        match self.advance()? {
            Instruction::PushBytes(bytes) => Ok(bytes),
            _ => Err(InscriptionError::InvalidInscription),
        }
    }

    fn accept(&mut self, instruction: Instruction) -> Result<bool> {
        match self.instructions.peek() {
            Some(Ok(next)) => {
                if *next == instruction {
                    self.advance()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Some(Err(err)) => Err(InscriptionError::Script(*err)),
            None => Ok(false),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_witness = &args[1..];

    let witness = Witness::from_vec(
        input_witness
            .iter()
            .map(|w| hex::decode(w).expect("Error decoding an input as hex"))
            .collect(),
    );

    if InscriptionParser::parse(&witness).is_ok() {
        println!("true");
    } else {
        println!("false");
    }
}

#[test]
/// Taken from https://www.blockchain.com/explorer/transactions/btc/6e995548e5be3c6f215f9301ae0d53691100b23ddaa4e5b12076503d5b1646ca
fn test_tx_decode() {
    let src = Witness::from_vec(vec![
      hex::decode("2467bd2b442cad0b6d03c8eca006a0b729cc7dd53c835537a9055e57bda5e154d7ba646477dfdedbbcb43d7e2653b05df248afca53a5682dad690b8a27ade45f").unwrap(),
      hex::decode("20117f692257b2331233b5705ce9c682be8719ff1b2b64cbca290bd6faeb54423eac06a7e609328801750063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800347b2270223a226272632d3230222c226f70223a226d696e74222c227469636b223a2250444159222c22616d74223a22353030227d68").unwrap(),
      hex::decode("c1117f692257b2331233b5705ce9c682be8719ff1b2b64cbca290bd6faeb54423e").unwrap()
    ]);

    let tx = InscriptionParser::parse(&src).unwrap(); //serde_json::from_str(src).unwrap();

    assert!(String::from_utf8(tx.body.unwrap())
        .unwrap()
        .contains("brc-20"));

    println!(
        "{:?}",
        String::from_utf8(tx.content_type.unwrap())
            .unwrap()
            .contains("charset")
    );
}
