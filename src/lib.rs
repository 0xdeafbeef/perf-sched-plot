use winnow::ascii::dec_uint;
use winnow::combinator::{alt, opt, separated_pair};
use winnow::{
    ascii::{multispace0, multispace1},
    combinator::delimited,
    token::{take_till, take_while},
    PResult, Parser,
};

pub struct ParsedData<'a> {
    pub time: f64,
    pub cpu: u16,
    pub task_name: &'a str,
    pub tid: u64,
    pub pid: Option<u64>,
    pub wait_time_ms: f64,
    pub sch_delay_ms: f64,
    pub run_time_ms: f64,
}

impl ParsedData<'_> {
    pub fn from_str<'a>(mut s: &'a str) -> PResult<ParsedData<'a>> {
        parse_line(&mut s)
    }
}

fn parse_cpu(input: &mut &str) -> PResult<u16> {
    delimited(
        '[',
        // Parse exactly 4 digits and convert to u16
        take_while(4..=4, |c: char| c.is_ascii_digit()).try_map(|s: &str| s.parse::<u16>()),
        ']',
    )
    .parse_next(input)
}

fn parse_task_name<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_till(1.., |c| c == '[')
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

fn parse_tid_pid(input: &mut &str) -> PResult<(u64, Option<u64>)> {
    let tid_pid_parser = separated_pair(dec_uint, '/', dec_uint.map(Some));

    let single_parser = dec_uint.map(|tid| (tid, None));

    let tid_pid_parser = alt((tid_pid_parser, single_parser));
    delimited('[', tid_pid_parser, ']').parse_next(input)
}

fn parse_float_value(input: &mut &str) -> PResult<f64> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == '.')
        .try_map(|s: &str| s.parse::<f64>())
        .parse_next(input)
}

fn parse_line<'a>(input: &mut &'a str) -> PResult<ParsedData<'a>> {
    take_while(0.., ' ').parse_next(input)?;
    // Time
    let time = parse_float_value.parse_next(input)?;
    multispace1.parse_next(input)?;

    // CPU
    let cpu = parse_cpu.parse_next(input)?;
    multispace1.parse_next(input)?;

    // Task name
    let task_name = parse_task_name.parse_next(input)?;

    // TID/PID
    let (tid, pid) = parse_tid_pid.parse_next(input)?;
    multispace0.parse_next(input)?;

    // Times
    let wait_time_ms = parse_float_value.parse_next(input)?;
    multispace1.parse_next(input)?;
    let sch_delay_ms = parse_float_value.parse_next(input)?;
    multispace1.parse_next(input)?;
    let run_time_ms = parse_float_value.parse_next(input)?;

    Ok(ParsedData {
        time,
        cpu,
        task_name,
        tid,
        wait_time_ms,
        sch_delay_ms,
        run_time_ms,
        pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_data() {
        let mut  s = " 2777601.141980 [0000]  migration/0[18]                     0.000      0.002      0.012";
        let parsed = parse_line(&mut s).unwrap();
        assert_eq!(parsed.time, 2777601.141980);
        assert_eq!(parsed.cpu, 0);
        assert_eq!(parsed.task_name, "migration/0");
        assert_eq!(parsed.tid, 18);
        assert_eq!(parsed.wait_time_ms, 0.000);
        assert_eq!(parsed.sch_delay_ms, 0.002);
        assert_eq!(parsed.run_time_ms, 0.012);

        let mut s = "2777601.142344 [0005]  tokio-runtime-w[426106/426078]      0.002      0.000      0.004";
        let parsed = parse_line(&mut s).unwrap();
        assert_eq!(parsed.time, 2777601.142344);
        assert_eq!(parsed.cpu, 5);
        assert_eq!(parsed.task_name, "tokio-runtime-w");
        assert_eq!(parsed.tid, 426106);
        assert_eq!(parsed.pid, Some(426078));
        assert_eq!(parsed.wait_time_ms, 0.002);
        assert_eq!(parsed.sch_delay_ms, 0.000);
        assert_eq!(parsed.run_time_ms, 0.004);
    }

    #[test]
    fn test_tid_pid() {
        let mut s = "[426106/426078]";
        let (tid, pid) = parse_tid_pid.parse(s).unwrap();
        assert_eq!(tid, 426106);
        assert_eq!(pid, Some(426078));

        let mut s = "[426106]";
        let (tid, pid) = parse_tid_pid.parse(s).unwrap();
        assert_eq!(tid, 426106);
        assert_eq!(pid, None);
    }
}
