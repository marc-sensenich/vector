use super::prelude::{error_stage, error_type};
use metrics::counter;
use vector_core::internal_event::InternalEvent;

use crate::sources::aws_kinesis_firehose::Compression;

#[derive(Debug)]
pub(crate) struct AwsKinesisFirehoseRequestReceived<'a> {
    pub(crate) request_id: Option<&'a str>,
    pub(crate) source_arn: Option<&'a str>,
}

impl<'a> InternalEvent for AwsKinesisFirehoseRequestReceived<'a> {
    fn emit_logs(&self) {
        info!(
            message = "Handling AWS Kinesis Firehose request.",
            request_id = %self.request_id.unwrap_or_default(),
            source_arn = %self.source_arn.unwrap_or_default(),
            internal_log_rate_secs = 10
        );
    }

    fn emit_metrics(&self) {
        counter!("requests_received_total", 1);
    }
}

#[derive(Debug)]
pub(crate) struct AwsKinesisFirehoseRequestError<'a> {
    pub(crate) request_id: Option<&'a str>,
    pub(crate) code: hyper::StatusCode,
    pub error: &'a str,
}

impl<'a> InternalEvent for AwsKinesisFirehoseRequestError<'a> {
    fn emit_logs(&self) {
        error!(
            message = "Error occurred while handling request.",
            error = ?self.error,
            error_type = error_type::REQUEST_FAILED,
            error_code = %self.code,
            stage = error_stage::RECEIVING,
            internal_log_rate_secs = 10
        );
    }

    fn emit_metrics(&self) {
        counter!(
            "component_errors_total", 1,
            "stage" => error_stage::RECEIVING,
            "error" => self.code.canonical_reason().unwrap_or("unknown status code"),
            "error_code" => self.code.to_string(),
            "error_type" => error_type::REQUEST_FAILED,
        );
        // deprecated
        counter!("request_read_errors_total", 1);
    }
}

#[derive(Debug)]
pub(crate) struct AwsKinesisFirehoseAutomaticRecordDecodeError {
    pub(crate) compression: Compression,
    pub(crate) error: std::io::Error,
}

impl InternalEvent for AwsKinesisFirehoseAutomaticRecordDecodeError {
    fn emit_logs(&self) {
        error!(
            message = %format!("Detected record as {} but failed to decode so passing along data as-is.", self.compression),
            error = ?self.error,
            stage = error_stage::PROCESSING,
            error_type = error_type::PARSER_FAILED,
            internal_log_rate_secs = 10
        );
    }

    fn emit_metrics(&self) {
        counter!(
            "component_errors_total", 1,
            "stage" => error_stage::PROCESSING,
            "error" => self.error.to_string(),
            "error_type" => error_type::PARSER_FAILED,
        );
        // deprecated
        counter!("request_automatic_decode_errors_total", 1);
    }
}
