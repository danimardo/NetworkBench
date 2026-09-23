#[cfg(test)]
mod tests {
    use crate::history::comparison::{
        COHORT_DIFFERENCE_THRESHOLD_PERCENT, COHORT_SAMPLE_LIMIT, evaluate_cohort_comparison,
        format_bps_human,
    };

    #[test]
    fn test_format_bps_human_conventions() {
        assert_eq!(format_bps_human(9_300_000_000), "9,3 Gbit/s");
        assert_eq!(format_bps_human(1_000_000_000), "1,0 Gbit/s");
        assert_eq!(format_bps_human(500_000_000), "500,0 Mbit/s");
        assert_eq!(format_bps_human(54_500_000), "54,5 Mbit/s");
    }

    #[test]
    fn test_cohort_five_completed_sessions_average_and_20_percent_threshold() {
        // 5 sesiones previas completadas con media exacta de 9.300.000.000 bps (9,3 Gbit/s)
        let past_sessions = vec![
            9_200_000_000,
            9_400_000_000,
            9_300_000_000,
            9_350_000_000,
            9_250_000_000,
        ];
        assert_eq!(past_sessions.len(), COHORT_SAMPLE_LIMIT);

        // 1. Caso rendimiento significativamente inferior (> 20 %)
        // 4.278.000.000 bps = 54 % inferior a 9,3 Gbit/s (§19.4)
        let current_low = 4_278_000_000;
        let res_low = evaluate_cohort_comparison(current_low, &past_sessions).unwrap();

        assert_eq!(res_low.sample_count, 5);
        assert_eq!(res_low.average_forward_bps, 9_300_000_000);
        assert!(res_low.is_significantly_lower);
        assert!(!res_low.is_significantly_higher);
        assert!(res_low.difference_percent > COHORT_DIFFERENCE_THRESHOLD_PERCENT);

        let obs = res_low
            .observation_text
            .expect("Debe haber observación de rendimiento inferior");
        assert!(obs.contains("inferior a la media reciente"));
        assert!(obs.contains("9,3 Gbit/s"));
        assert!(obs.contains("54 %"));

        // 2. Caso rendimiento significativamente superior (> 20 %)
        // 11.625.000.000 bps = 25 % superior a 9,3 Gbit/s
        let current_high = 11_625_000_000;
        let res_high = evaluate_cohort_comparison(current_high, &past_sessions).unwrap();

        assert!(res_high.is_significantly_higher);
        assert!(!res_high.is_significantly_lower);
        let obs_high = res_high
            .observation_text
            .expect("Debe haber observación de rendimiento superior");
        assert!(obs_high.contains("superior a la media reciente"));
        assert!(obs_high.contains("25 %"));

        // 3. Caso rendimiento dentro del margen esperado (<= 20 % de diferencia)
        // 9.000.000.000 bps = ~3.2 % de diferencia
        let current_normal = 9_000_000_000;
        let res_normal = evaluate_cohort_comparison(current_normal, &past_sessions).unwrap();

        assert!(!res_normal.is_significantly_lower);
        assert!(!res_normal.is_significantly_higher);
        assert_eq!(res_normal.observation_text, None);
    }

    #[test]
    fn test_cohort_limits_to_five_recent_samples() {
        // Si hay 10 sesiones en la historia, solo se promedian las 5 más recientes
        let past_ten = vec![
            10_000_000_000,
            10_000_000_000,
            10_000_000_000,
            10_000_000_000,
            10_000_000_000, // primeras 5 tienen media 10G
            1_000_000_000,
            1_000_000_000,
            1_000_000_000,
            1_000_000_000,
            1_000_000_000, // antiguas tienen 1G
        ];

        let res = evaluate_cohort_comparison(10_000_000_000, &past_ten).unwrap();
        assert_eq!(res.sample_count, 5);
        assert_eq!(res.average_forward_bps, 10_000_000_000);
        assert!(!res.is_significantly_lower);
    }

    #[test]
    fn test_cohort_empty_samples_returns_none() {
        let empty: Vec<u64> = vec![];
        assert_eq!(evaluate_cohort_comparison(9_000_000_000, &empty), None);
    }
}
