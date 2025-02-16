pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct Object {
        pub id: u64,
        pub description: String,
        pub kind: ObjectKind,
        pub num_sprites: u64,
        pub sprites: Vec<Sprite>,
        pub head_index: Vec<i64>,
        pub body_index: Vec<i64>,
        pub back_foot_index: Vec<i64>,
        pub front_foot_index: Vec<i64>,
    }

    #[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum ObjectKind {
        Person(PersonCharacteristic),

        NonPerson(NonPersonObject),
    }

    impl Default for ObjectKind {
        fn default() -> Self {
            Self::NonPerson(NonPersonObject::default())
        }
    }

    #[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum ClothingObject {
        Shoe(Position),
        Tunic(Position),
        Hat(Position),
        Bottom(Position),
        Backpack(Position),
    }

    impl Default for ClothingObject {
        fn default() -> Self {
            Self::Tunic(Position::default())
        }
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub enum NonPersonObject {
        Clothing(ClothingObject),
        #[default]
        Other,
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub enum PersonCharacteristic {
        #[default]
        Feminine,
        Masculine,
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct Sprite {
        pub id: u64,
        pub position: Position,
        pub rot: Number,
        pub h_flip: Number,
        pub color: ColorRGB,
        pub age_range: AgeRange,
        pub parent: i64,
        pub invis_holding: Number,
        pub invis_worn: Number,
        pub behind_slots: Number,
        pub invis_cont: Option<Number>,
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct Position {
        pub x: Number,
        pub y: Number,
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct Number(pub f64);

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct ColorRGB {
        pub r: Number,
        pub g: Number,
        pub b: Number,
    }

    #[derive(
        Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct AgeRange {
        pub min: Number,
        pub max: Number,
    }
}

use std::{fs, path::PathBuf, vec};

use winnow::{
    ascii::{alphanumeric1, dec_int, dec_uint, float, line_ending},
    combinator::{alt, opt, repeat_till, separated},
    error::{ContextError, ParserError},
    stream::{Compare, Stream, StreamIsPartial},
    token::{literal, none_of, take_until},
    Parser, Result,
};

use types::{
    AgeRange, ClothingObject, ColorRGB, NonPersonObject, Number, Object,
    ObjectKind, PersonCharacteristic, Position, Sprite,
};

pub fn parse(objects_dir: &PathBuf) -> anyhow::Result<Vec<Object>> {
    let mut objects = vec![];
    for entry in fs::read_dir(objects_dir)? {
        let entry = entry?;
        let path = entry.path();
        let non_object_files = vec![
            "nextObjectNumber.txt",
            "groundHeat_6.txt",
            "groundHeat_5.txt",
            "groundHeat_4.txt",
        ];
        let is_object_file =
            !non_object_files.iter().any(|f| path == PathBuf::from(f));

        if let Some(ext) = path.extension() {
            if ext == "txt" && is_object_file {
                let content = fs::read_to_string(path)?;

                if let Ok(obj) = parse_object(&mut content.as_str()) {
                    objects.push(obj);
                }
            }
        }
    }
    Ok(objects)
}

fn parse_object(input: &mut &str) -> Result<Object> {
    let id: u64 = parse_assignment(input, "id", dec_uint)?;
    line_ending(input)?;

    let (description, _): (String, &str) =
        repeat_till(0.., none_of::<_, _, ContextError>(['\n']), line_ending)
            .parse_next(input)?;

    take_until(0.., "person").parse_next(input)?; // skip the rest after

    let person: u8 = parse_assignment(input, "person", dec_uint)?;
    let is_person = person > 0;

    take_until(0.., "male").parse_next(input)?;

    let male: u8 = parse_assignment(input, "male", dec_uint)?;
    let is_male = male > 0;

    take_until(0.., "clothing").parse_next(input)?;

    let clothing = parse_assignment(input, "clothing", alphanumeric1)?;
    let is_clothing = clothing != "n";
    separator(input)?;
    let clothing_offset =
        parse_assignment(input, "clothingOffset", |i: &mut &str| {
            let x = parse_number.parse_next(i)?;
            ','.parse_next(i)?;
            let y = parse_number.parse_next(i)?;

            Ok(Position { x, y })
        })?;

    let kind = if is_person {
        if is_male {
            ObjectKind::Person(PersonCharacteristic::Masculine)
        } else {
            ObjectKind::Person(PersonCharacteristic::Feminine)
        }
    } else if is_clothing {
        ObjectKind::NonPerson(NonPersonObject::Clothing(match clothing {
            "s" => ClothingObject::Shoe(clothing_offset),
            "t" => ClothingObject::Tunic(clothing_offset),
            "h" => ClothingObject::Hat(clothing_offset),
            "b" => ClothingObject::Bottom(clothing_offset),
            _ => ClothingObject::default(),
        }))
    } else {
        ObjectKind::NonPerson(NonPersonObject::Other)
    };

    take_until(0.., "numSprites").parse_next(input)?;
    let num_sprites: u64 = parse_assignment(input, "numSprites", dec_uint)?;
    separator(input)?;
    let (sprites, head_index) = parse_sprites(input)?;
    separator(input)?;
    let body_index = parse_assignment(input, "bodyIndex", parse_index_list)?;
    separator(input)?;

    let back_foot_index =
        parse_assignment(input, "backFootIndex", parse_index_list)?;
    separator(input)?;
    let front_foot_index =
        parse_assignment(input, "frontFootIndex", parse_index_list)?;

    Ok(Object {
        id,
        description,
        kind,
        num_sprites,
        sprites,
        head_index,
        body_index,
        back_foot_index,
        front_foot_index,
    })
}

#[cfg(test)]
mod parse_object_tests {

    use std::vec;

    use winnow::Parser;

    use crate::parser::{
        parse_object,
        types::{
            AgeRange, ColorRGB, Number, Object, ObjectKind,
            PersonCharacteristic, Position, Sprite,
        },
    };

    #[test]
    fn test() {
        let source = "id=1
wth is going on here?? meowi! spzz **@#HJasba sa whs
person=1,spawn=0
male=0
clothing=n
clothingOffset=0.2,4.0
numSprites=7
spriteID=110013
pos=40.000000,-23.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=-12.000000,-9.000000
rot=0.000000
hFlip=1
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=-39.000000,-16.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=493
pos=1.000000,-35.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=16.000000,-12.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
headIndex=-1
bodyIndex=4,9,12,1
backFootIndex=9,19,22,33,39
frontFootIndex=6,15,17,30,36";
        assert_eq!(
            parse_object.parse_peek(source),
            Ok((
                "",
                Object {
                    id: 1,
                    description:
                        "wth is going on here?? meowi! spzz **@#HJasba sa whs"
                            .to_string(),
                    kind: ObjectKind::Person(PersonCharacteristic::Feminine),
                    num_sprites: 7,
                    sprites: vec![
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(40.0),
                                y: Number(-23.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(-12.0),
                                y: Number(-9.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(1.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(-39.0),
                                y: Number(-16.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 493,
                            position: Position {
                                x: Number(1.0),
                                y: Number(-35.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(16.0),
                                y: Number(-12.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        }
                    ],
                    head_index: vec![-1],
                    body_index: vec![4, 9, 12, 1],
                    back_foot_index: vec![9, 19, 22, 33, 39],
                    front_foot_index: vec![6, 15, 17, 30, 36],
                }
            ))
        );
    }
}

fn separator<'a>(input: &mut &'a str) -> Result<&'a str> {
    alt((line_ending, ",")).parse_next(input)
}

fn parse_sprites<'a>(input: &mut &'a str) -> Result<(Vec<Sprite>, Vec<i64>)> {
    let parse_sprite_le = |i: &mut &'a str| {
        let sprite = parse_sprite(i)?;
        separator(i)?;

        Ok(sprite)
    };
    let terminator =
        |i: &mut &'a str| parse_assignment(i, "headIndex", parse_index_list);
    let (sprites, head_index): (Vec<Sprite>, Vec<i64>) =
        repeat_till(0.., parse_sprite_le, terminator).parse_next(input)?;

    Ok((sprites, head_index))
}

fn parse_index_list(input: &mut &str) -> Result<Vec<i64>> {
    let (first_elem, has_many) =
        (dec_int::<_, i64, _>, opt(",")).parse_next(input)?;

    match has_many {
        Some(_) => {
            let elems: Vec<i64> = separated(0.., dec_int::<_, i64, _>, ",")
                .parse_next(input)?;

            Ok([vec![first_elem], elems].concat())
        }
        None => Ok(vec![first_elem]),
    }
}

#[cfg(test)]
mod test_parse_index_list {
    use winnow::Parser;

    use crate::parser::parse_index_list;

    #[test]
    fn test() {
        assert_eq!(
            parse_index_list.parse_peek("1,3,5,9,2"),
            Ok(("", vec![1, 3, 5, 9, 2]))
        );
    }
}

#[cfg(test)]
mod test_parse_sprites {
    use winnow::Parser;

    use crate::parser::{
        parse_sprites,
        types::{AgeRange, ColorRGB, Number, Position, Sprite},
    };

    #[test]
    fn test() {
        let source = "spriteID=110013
pos=40.000000,-23.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=-12.000000,-9.000000
rot=0.000000
hFlip=1
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=-39.000000,-16.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=493
pos=1.000000,-35.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
spriteID=110013
pos=16.000000,-12.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0
headIndex=-1";
        assert_eq!(
            parse_sprites.parse_peek(source),
            Ok((
                "",
                (
                    vec![
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(40.0),
                                y: Number(-23.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(-12.0),
                                y: Number(-9.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(1.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(-39.0),
                                y: Number(-16.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 493,
                            position: Position {
                                x: Number(1.0),
                                y: Number(-35.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        },
                        Sprite {
                            id: 110013,
                            position: Position {
                                x: Number(16.0),
                                y: Number(-12.0)
                            },
                            rot: Number(0.0),
                            h_flip: Number(0.0),
                            color: ColorRGB {
                                r: Number(1.0),
                                g: Number(1.0),
                                b: Number(1.0)
                            },
                            age_range: AgeRange {
                                min: Number(-1.0),
                                max: Number(-1.0)
                            },
                            parent: -1,
                            invis_holding: Number(0.0),
                            invis_worn: Number(0.0),
                            behind_slots: Number(0.0),
                            invis_cont: Some(Number(0.0))
                        }
                    ],
                    vec![-1]
                )
            ))
        );
    }
}

fn parse_sprite<'a>(input: &mut &'a str) -> Result<Sprite> {
    let separator = |i: &mut &'a str| alt((line_ending, ",")).parse_next(i);
    let id: u64 = parse_assignment(input, "spriteID", dec_uint)?;
    separator(input)?;
    let position = parse_sprite_position(input)?;
    separator(input)?;
    let rot = parse_assignment(input, "rot", parse_number)?;
    separator(input)?;
    let h_flip = parse_assignment(input, "hFlip", parse_number)?;
    separator(input)?;
    let color = parse_assignment(input, "color", parse_sprite_color)?;
    separator(input)?;
    let age_range =
        parse_assignment(input, "ageRange", |i: &mut &'a str| {
            let (min, _, max) =
                (parse_number, ",", parse_number).parse_next(i)?;

            Ok(AgeRange { min, max })
        })?;
    separator(input)?;
    let parent = parse_assignment(input, "parent", dec_int)?;
    separator(input)?;
    let invis_holding =
        parse_assignment(input, "invisHolding", parse_number)?;
    separator(input)?;
    let invis_worn = parse_assignment(input, "invisWorn", parse_number)?;
    separator(input)?;
    let behind_slots = parse_assignment(input, "behindSlots", parse_number)?;
    opt(separator).parse_next(input)?;
    let invis_cont = opt(parse_invis_cont).parse_next(input)?;

    Ok(Sprite {
        id,
        position,
        rot,
        h_flip,
        color,
        age_range,
        parent,
        invis_holding,
        invis_worn,
        behind_slots,
        invis_cont,
    })
}

fn parse_invis_cont(input: &mut &str) -> Result<Number> {
    parse_assignment(input, "invisCont", parse_number)
}

fn parse_sprite_color(input: &mut &str) -> Result<ColorRGB> {
    let r = parse_number(input)?;
    ",".parse_next(input)?;
    let g = parse_number(input)?;
    ",".parse_next(input)?;
    let b = parse_number(input)?;

    Ok(ColorRGB { r, g, b })
}

fn parse_sprite_position<'a>(input: &mut &'a str) -> Result<Position> {
    parse_assignment(input, "pos", |i: &mut &'a str| {
        let x = parse_number.parse_next(i)?;
        ','.parse_next(i)?;
        let y = parse_number.parse_next(i)?;

        Ok(Position { x, y })
    })
}

#[cfg(test)]
mod sprite_parser_tests {
    use winnow::Parser;

    use crate::parser::{
        parse_sprite,
        types::{AgeRange, ColorRGB, Number, Position, Sprite},
    };

    #[test]
    fn test() {
        let source = "spriteID=1176
pos=-2.000000,-31.000000
rot=0.000000
hFlip=0
color=1.000000,1.000000,1.000000
ageRange=-1.000000,-1.000000
parent=-1
invisHolding=0,invisWorn=0,behindSlots=0
invisCont=0";
        assert_eq!(
            parse_sprite.parse_peek(source),
            Ok((
                "",
                Sprite {
                    id: 1176,
                    position: Position {
                        x: Number(-2.0),
                        y: Number(-31.0)
                    },
                    rot: Number(0.0),
                    h_flip: Number(0.0),
                    color: ColorRGB {
                        r: Number(1.0),
                        g: Number(1.0),
                        b: Number(1.0),
                    },
                    age_range: AgeRange {
                        min: Number(-1.0),
                        max: Number(-1.0)
                    },
                    parent: -1,
                    invis_holding: Number(0.0),
                    invis_worn: Number(0.0),
                    behind_slots: Number(0.0),
                    invis_cont: Some(Number(0.0))
                }
            ))
        );
    }
}

fn parse_number(input: &mut &str) -> Result<Number> {
    Ok(Number(float(input)?))
}

fn parse_assignment<I, O, E, P>(
    input: &mut I,
    key: &str,
    mut p: P,
) -> Result<O, E>
where
    I: Stream + StreamIsPartial + for<'a> Compare<&'a str>,
    E: ParserError<I>,
    P: Parser<I, O, E>,
{
    literal(key).parse_next(input)?;
    "=".parse_next(input)?;

    p.parse_next(input)
}
