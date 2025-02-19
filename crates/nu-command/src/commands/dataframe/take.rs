use crate::prelude::*;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{
    dataframe::{Column, NuDataFrame},
    Signature, SyntaxShape, UntaggedValue, Value,
};
use polars::prelude::DataType;

use super::utils::parse_polars_error;

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "dataframe take"
    }

    fn usage(&self) -> &str {
        "[DataFrame, Series] Creates new dataframe using the given indices"
    }

    fn signature(&self) -> Signature {
        Signature::build("dataframe take").required(
            "indices",
            SyntaxShape::Any,
            "list of indices used to take data",
        )
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Takes selected rows from dataframe",
                example: r#"let df = ([[a b]; [4 1] [5 2] [4 3]] | dataframe to-df);
    let indices = ([0 2] | dataframe to-df);
    $df | dataframe take $indices"#,
                result: Some(vec![NuDataFrame::try_from_columns(
                    vec![
                        Column::new(
                            "a".to_string(),
                            vec![UntaggedValue::int(4).into(), UntaggedValue::int(4).into()],
                        ),
                        Column::new(
                            "b".to_string(),
                            vec![UntaggedValue::int(1).into(), UntaggedValue::int(3).into()],
                        ),
                    ],
                    &Span::default(),
                )
                .expect("simple df for test should not fail")
                .into_value(Tag::default())]),
            },
            Example {
                description: "Takes selected rows from series",
                example: r#"let series = ([4 1 5 2 4 3] | dataframe to-df);
    let indices = ([0 2] | dataframe to-df);
    $series | dataframe take $indices"#,
                result: Some(vec![NuDataFrame::try_from_columns(
                    vec![Column::new(
                        "0".to_string(),
                        vec![UntaggedValue::int(4).into(), UntaggedValue::int(5).into()],
                    )],
                    &Span::default(),
                )
                .expect("simple df for test should not fail")
                .into_value(Tag::default())]),
            },
        ]
    }
}

fn command(mut args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();
    let value: Value = args.req(0)?;

    let df = match &value.value {
        UntaggedValue::DataFrame(df) => Ok(df),
        _ => Err(ShellError::labeled_error(
            "Incorrect type",
            "can only use a series for take command",
            value.tag.span,
        )),
    }?;

    let series = df.as_series(&value.tag.span)?;

    let casted = match series.dtype() {
        DataType::UInt32 | DataType::UInt64 | DataType::Int32 | DataType::Int64 => series
            .as_ref()
            .cast_with_dtype(&DataType::UInt32)
            .map_err(|e| parse_polars_error::<&str>(&e, &value.tag.span, None)),
        _ => Err(ShellError::labeled_error_with_secondary(
            "Incorrect type",
            "Series with incorrect type",
            &value.tag.span,
            "Consider using a Series with type int type",
            &value.tag.span,
        )),
    }?;

    let indices = casted
        .u32()
        .map_err(|e| parse_polars_error::<&str>(&e, &value.tag.span, None))?;

    let value = args.input.next().ok_or_else(|| {
        ShellError::labeled_error("Empty stream", "No value found in the stream", &tag)
    })?;

    match value.value {
        UntaggedValue::DataFrame(df) => {
            let res = df.as_ref().take(indices);

            Ok(OutputStream::one(NuDataFrame::dataframe_to_value(res, tag)))
        }
        _ => Err(ShellError::labeled_error(
            "No dataframe or series in stream",
            "no dataframe or series found in input stream",
            &value.tag.span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::DataFrame;
    use super::ShellError;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test_dataframe as test_examples;

        test_examples(DataFrame {})
    }
}
