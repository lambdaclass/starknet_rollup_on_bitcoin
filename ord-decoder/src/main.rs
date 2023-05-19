use std::{println, iter::Peekable, collections::BTreeMap};

use bitcoin::{blockdata::{script::{Instructions, Instruction, self}, opcodes}, Witness, util::taproot::TAPROOT_ANNEX_PREFIX, Script};

fn main() {
    let src = Witness::from_vec(vec![
        hex::decode("2467bd2b442cad0b6d03c8eca006a0b729cc7dd53c835537a9055e57bda5e154d7ba646477dfdedbbcb43d7e2653b05df248afca53a5682dad690b8a27ade45f").unwrap(),
        hex::decode("20117f692257b2331233b5705ce9c682be8719ff1b2b64cbca290bd6faeb54423eac06a7e609328801750063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800347b2270223a226272632d3230222c226f70223a226d696e74222c227469636b223a2250444159222c22616d74223a22353030227d68").unwrap()
        ,hex::decode("c1117f692257b2331233b5705ce9c682be8719ff1b2b64cbca290bd6faeb54423e").unwrap()
    ]);
    let tx= InscriptionParser::parse(&src);    //serde_json::from_str(src).unwrap();
    println!("{:?}", tx);
}

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
    self
      .instructions
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
