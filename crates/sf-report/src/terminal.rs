use sf_core::{Artifact, StatsKind};

/// Render a single artifact to terminal.
pub fn render(name: &str, artifact: &Artifact) {
    println!("=== {} ===", name);
    match artifact {
        Artifact::Stats(stats) => render_stats(stats),
        Artifact::Dataset(ds) => {
            if let Ok(df) = ds.collect() {
                println!("{}", df);
            }
        }
        Artifact::Table(df) => {
            println!("{}", df);
        }
        Artifact::Text(text) => {
            println!("{}", text);
        }
        Artifact::Plot(plot) => {
            println!("[SVG plot: {}] ({} bytes)", plot.title, plot.svg.len());
        }
        Artifact::Forecast(fc) => {
            println!("Model: {}", fc.model);
            println!(
                "{:>10} {:>12} {:>12} {:>12}",
                "Index", "Yhat", "Lower", "Upper"
            );
            for pt in &fc.points {
                println!(
                    "{:>10} {:>12.4} {:>12} {:>12}",
                    pt.index,
                    pt.yhat,
                    pt.lower.map_or("N/A".to_string(), |v| format!("{:.4}", v)),
                    pt.upper.map_or("N/A".to_string(), |v| format!("{:.4}", v)),
                );
            }
        }
        Artifact::Model(m) => {
            println!("[Model: {}] ({} bytes)", m.name, m.data.len());
        }
    }
    println!();
}

fn render_stats(stats: &sf_core::StatsArtifact) {
    match stats.kind {
        StatsKind::Summary => render_summary(stats),
        StatsKind::Correlation => render_correlation(stats),
        StatsKind::Distribution => render_distribution(stats),
        StatsKind::OutlierDetection => render_outlier(stats),
        StatsKind::Regression => render_regression(stats),
        StatsKind::RegressionDiagnostics => render_regression_diagnostics(stats),
        StatsKind::Adf => render_adf(stats),
        StatsKind::Decompose => render_decompose(stats),
        StatsKind::ArimaForecast => render_generic(stats),
        StatsKind::TTest => render_ttest(stats),
        StatsKind::ConfidenceInterval => render_ci(stats),
        StatsKind::ProportionTest => render_proportion_test(stats),
        StatsKind::ChiSquareTest => render_chi_square(stats),
        StatsKind::VarianceTest => render_variance_test(stats),
        _ => render_generic(stats),
    }
}

fn render_summary(stats: &sf_core::StatsArtifact) {
    if let Some(columns) = stats.values.get("columns") {
        if let Some(arr) = columns.as_array() {
            println!(
                "{:<15} {:>10} {:>10} {:>12} {:>12} {:>12} {:>12} {:>10} {:>10} {:>10}",
                "Column", "Count", "Nulls", "Mean", "Std", "Min", "Max", "Q25", "Q50", "Q75"
            );
            println!("{}", "-".repeat(125));
            for col in arr {
                let name = col.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let count = col.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                let nulls = col.get("null_count").and_then(|v| v.as_u64()).unwrap_or(0);
                println!(
                    "{:<15} {:>10} {:>10} {:>12} {:>12} {:>12} {:>12} {:>10} {:>10} {:>10}",
                    name,
                    count,
                    nulls,
                    fmt_opt_val(col.get("mean")),
                    fmt_opt_val(col.get("std")),
                    fmt_opt_val(col.get("min")),
                    fmt_opt_val(col.get("max")),
                    fmt_opt_val(col.get("q25")),
                    fmt_opt_val(col.get("q50")),
                    fmt_opt_val(col.get("q75")),
                );
            }
        }
    }
}

fn render_correlation(stats: &sf_core::StatsArtifact) {
    let method = stats
        .values
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let columns = stats
        .values
        .get("columns")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let matrix = stats
        .values
        .get("matrix")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    print!("{:<15}", "");
    for col in &columns {
        print!("{:>12}", col);
    }
    println!();
    for (i, row) in matrix.iter().enumerate() {
        if let Some(row_arr) = row.as_array() {
            print!("{:<15}", columns.get(i).map_or("?", |s| s.as_str()));
            for val in row_arr {
                print!("{:>12.4}", val.as_f64().unwrap_or(0.0));
            }
            println!();
        }
    }
    println!("Method: {}", method);
}

fn render_distribution(stats: &sf_core::StatsArtifact) {
    let column = stats
        .values
        .get("column")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    println!("Column: {}", column);
    if let Some(bins) = stats.values.get("bins").and_then(|v| v.as_array()) {
        println!("{:>12} {:>12} {:>10}", "Lower", "Upper", "Count");
        println!("{}", "-".repeat(36));
        for bin in bins {
            let lower = bin.get("lower").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let upper = bin.get("upper").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let count = bin.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("{:>12.4} {:>12.4} {:>10}", lower, upper, count);
        }
    }
}

fn render_outlier(stats: &sf_core::StatsArtifact) {
    for (key, val) in &stats.values {
        if let Some(obj) = val.as_object() {
            println!("Column: {}", key);
            println!(
                "  Method:       {}",
                obj.get("method").and_then(|v| v.as_str()).unwrap_or("?")
            );
            println!(
                "  Total rows:   {}",
                obj.get("total_rows").and_then(|v| v.as_u64()).unwrap_or(0)
            );
            println!(
                "  Outliers:     {}",
                obj.get("outlier_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            );
            println!(
                "  Bounds:       [{:.4}, {:.4}]",
                obj.get("lower_bound")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                obj.get("upper_bound")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            );
        }
    }
}

fn render_regression(stats: &sf_core::StatsArtifact) {
    let target = stats
        .values
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let r2 = stats
        .values
        .get("r_squared")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let adj_r2 = stats
        .values
        .get("adj_r_squared")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let f_stat = stats
        .values
        .get("f_statistic")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let n = stats.values.get("n").and_then(|v| v.as_u64()).unwrap_or(0);
    let df_model = stats
        .values
        .get("df_model")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let df_resid = stats
        .values
        .get("df_resid")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let conf = stats
        .values
        .get("confidence_level")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.95);

    println!("OLS Regression: {}", target);
    println!(
        "  R² = {:.6}  Adj R² = {:.6}  F = {:.4}  n = {}",
        r2, adj_r2, f_stat, n
    );
    println!("  df_model = {}  df_resid = {}", df_model, df_resid);
    println!();

    if let Some(coefficients) = stats.values.get("coefficients").and_then(|v| v.as_array()) {
        println!(
            "{:<15} {:>12} {:>12} {:>10} {:>12} {:>12} {:>12}",
            "Variable",
            "Estimate",
            "Std.Error",
            "t value",
            "p value",
            &format!("CI {:.0}% Lo", conf * 100.0),
            &format!("CI {:.0}% Hi", conf * 100.0),
        );
        println!("{}", "-".repeat(97));
        for coeff in coefficients {
            let name = coeff.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let est = coeff
                .get("estimate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let se = coeff
                .get("std_error")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let t = coeff.get("t_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let p = coeff.get("p_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let ci_lo = coeff
                .get("ci_lower")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let ci_hi = coeff
                .get("ci_upper")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let sig = if p < 0.001 {
                "***"
            } else if p < 0.01 {
                "**"
            } else if p < 0.05 {
                "*"
            } else if p < 0.1 {
                "."
            } else {
                ""
            };

            println!(
                "{:<15} {:>12.6} {:>12.6} {:>10.4} {:>12.6} {:>12.6} {:>12.6} {}",
                name, est, se, t, p, ci_lo, ci_hi, sig
            );
        }
        println!("---");
        println!("Signif. codes:  0 '***' 0.001 '**' 0.01 '*' 0.05 '.' 0.1 ' ' 1");
    }
}

fn render_regression_diagnostics(stats: &sf_core::StatsArtifact) {
    let target = stats
        .values
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    println!("Regression Diagnostics: {}", target);
    println!();

    // Durbin-Watson
    if let Some(dw) = stats.values.get("durbin_watson").and_then(|v| v.as_f64()) {
        println!("Durbin-Watson: {:.4}", dw);
    }

    // VIF
    if let Some(vif_arr) = stats.values.get("vif").and_then(|v| v.as_array()) {
        println!();
        println!("{:<15} {:>10}", "Predictor", "VIF");
        println!("{}", "-".repeat(27));
        for entry in vif_arr {
            let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let vif = entry.get("vif").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let flag = if vif > 10.0 {
                " !!!"
            } else if vif > 5.0 {
                " !"
            } else {
                ""
            };
            println!("{:<15} {:>10.4}{}", name, vif, flag);
        }
    }

    // Residual stats
    if let Some(rs) = stats.values.get("residual_stats") {
        println!();
        println!("Residual Statistics:");
        println!(
            "  Mean:     {:.6}",
            rs.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Std:      {:.6}",
            rs.get("std").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Min:      {:.6}",
            rs.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Max:      {:.6}",
            rs.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Skewness: {:.6}",
            rs.get("skewness").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Kurtosis: {:.6}",
            rs.get("kurtosis").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
    }

    // Leverage
    if let Some(lev) = stats.values.get("leverage") {
        println!();
        println!("Leverage:");
        println!(
            "  Mean:             {:.6}",
            lev.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Max:              {:.6}",
            lev.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Threshold (2p/n): {:.6}",
            lev.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  High leverage:    {}",
            lev.get("high_leverage_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        );
    }

    // Cook's distance
    if let Some(cd) = stats.values.get("cooks_distance") {
        println!();
        println!("Cook's Distance:");
        println!(
            "  Max:            {:.6}",
            cd.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Threshold (4/n):{:.6}",
            cd.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  Influential:    {}",
            cd.get("influential_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        );
    }
}

fn render_adf(stats: &sf_core::StatsArtifact) {
    println!("Augmented Dickey-Fuller Test");
    println!("{}", "-".repeat(40));

    let t_stat = stats
        .values
        .get("test_statistic")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let lag = stats
        .values
        .get("lag_used")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let n_obs = stats
        .values
        .get("n_obs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let is_stationary = stats
        .values
        .get("is_stationary_5pct")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("Test Statistic: {:.4}", t_stat);
    println!("Lags Used:      {}", lag);
    println!("Observations:   {}", n_obs);

    if let Some(cv) = stats.values.get("critical_values") {
        println!("Critical Values:");
        println!(
            "  1%:  {:.4}",
            cv.get("1%").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  5%:  {:.4}",
            cv.get("5%").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
        println!(
            "  10%: {:.4}",
            cv.get("10%").and_then(|v| v.as_f64()).unwrap_or(0.0)
        );
    }

    println!(
        "Result:         {}",
        if is_stationary {
            "Stationary (reject H0)"
        } else {
            "Non-stationary (fail to reject H0)"
        }
    );
}

fn render_decompose(stats: &sf_core::StatsArtifact) {
    let period = stats
        .values
        .get("period")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let method = stats
        .values
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let n_obs = stats
        .values
        .get("n_obs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("Time Series Decomposition");
    println!("{}", "-".repeat(40));
    println!("Method:       {}", method);
    println!("Period:       {}", period);
    println!("Observations: {}", n_obs);

    if let Some(summary) = stats.values.get("summary") {
        println!("Components:");
        println!(
            "  Trend mean:       {:.4}",
            summary
                .get("trend_mean")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        );
        println!(
            "  Trend std:        {:.4}",
            summary
                .get("trend_std")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        );
        println!(
            "  Seasonal range:   {:.4}",
            summary
                .get("seasonal_range")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        );
        println!(
            "  Residual mean:    {:.4}",
            summary
                .get("residual_mean")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        );
        println!(
            "  Residual std:     {:.4}",
            summary
                .get("residual_std")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        );
    }
}

fn render_ttest(stats: &sf_core::StatsArtifact) {
    let test = stats
        .values
        .get("test")
        .and_then(|v| v.as_str())
        .unwrap_or("t_test");
    println!("Test: {}", test);

    if let Some(col) = stats.values.get("column").and_then(|v| v.as_str()) {
        println!("Column: {}", col);
    }
    if let Some(group_col) = stats.values.get("group_column").and_then(|v| v.as_str()) {
        println!("Group column: {}", group_col);
    }

    println!();
    println!("{:<20} {:>15}", "Statistic", "Value");
    println!("{}", "-".repeat(37));

    let fields = [
        ("n", "N"),
        ("n1", "N₁"),
        ("n2", "N₂"),
        ("mean", "Mean"),
        ("mean1", "Mean₁"),
        ("mean2", "Mean₂"),
        ("std_dev", "Std Dev"),
        ("mu0", "μ₀"),
        ("mean_diff", "Mean Diff"),
        ("t_statistic", "t-statistic"),
        ("df", "df"),
        ("p_value", "p-value"),
        ("se", "Std Error"),
        ("effect_size", "Effect Size"),
    ];

    for (key, label) in &fields {
        if let Some(val) = stats.values.get(*key) {
            println!("{:<20} {:>15}", label, fmt_opt_val(Some(val)));
        }
    }

    if let Some(es_type) = stats
        .values
        .get("effect_size_type")
        .and_then(|v| v.as_str())
    {
        println!("{:<20} {:>15}", "Effect Size Type", es_type);
    }
    if let Some(alt) = stats.values.get("alternative").and_then(|v| v.as_str()) {
        println!("{:<20} {:>15}", "Alternative", alt);
    }

    println!();
    if let (Some(ci_lo), Some(ci_hi), Some(conf)) = (
        stats.values.get("ci_lower").and_then(|v| v.as_f64()),
        stats.values.get("ci_upper").and_then(|v| v.as_f64()),
        stats
            .values
            .get("confidence_level")
            .and_then(|v| v.as_f64()),
    ) {
        println!("{:.0}% CI: [{:.6}, {:.6}]", conf * 100.0, ci_lo, ci_hi);
    }
}

fn render_ci(stats: &sf_core::StatsArtifact) {
    let estimate = stats
        .values
        .get("estimate")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    println!("Confidence Interval for: {}", estimate);

    if let Some(col) = stats.values.get("column").and_then(|v| v.as_str()) {
        println!("Column: {}", col);
    }

    println!();
    println!("{:<20} {:>15}", "Statistic", "Value");
    println!("{}", "-".repeat(37));

    let fields = [
        ("n", "N"),
        ("successes", "Successes"),
        ("mean", "Mean"),
        ("p_hat", "p̂"),
        ("variance", "Variance"),
        ("std_dev", "Std Dev"),
        ("se", "Std Error"),
        ("df", "df"),
        ("margin_of_error", "Margin of Error"),
    ];

    for (key, label) in &fields {
        if let Some(val) = stats.values.get(*key) {
            println!("{:<20} {:>15}", label, fmt_opt_val(Some(val)));
        }
    }

    if let Some(method) = stats.values.get("method").and_then(|v| v.as_str()) {
        println!("{:<20} {:>15}", "Method", method);
    }

    println!();
    if let (Some(ci_lo), Some(ci_hi), Some(conf)) = (
        stats.values.get("ci_lower").and_then(|v| v.as_f64()),
        stats.values.get("ci_upper").and_then(|v| v.as_f64()),
        stats
            .values
            .get("confidence_level")
            .and_then(|v| v.as_f64()),
    ) {
        println!("{:.0}% CI: [{:.6}, {:.6}]", conf * 100.0, ci_lo, ci_hi);
    }
}

fn render_proportion_test(stats: &sf_core::StatsArtifact) {
    println!("Test: Proportion z-test");
    println!();
    println!("{:<20} {:>15}", "Statistic", "Value");
    println!("{}", "-".repeat(37));

    let fields = [
        ("n", "N"),
        ("successes", "Successes"),
        ("p_hat", "p̂"),
        ("p0", "p₀"),
        ("z_statistic", "z-statistic"),
        ("p_value", "p-value"),
        ("se", "Std Error"),
        ("effect_size", "Effect Size"),
    ];

    for (key, label) in &fields {
        if let Some(val) = stats.values.get(*key) {
            println!("{:<20} {:>15}", label, fmt_opt_val(Some(val)));
        }
    }

    if let Some(es_type) = stats
        .values
        .get("effect_size_type")
        .and_then(|v| v.as_str())
    {
        println!("{:<20} {:>15}", "Effect Size Type", es_type);
    }
    if let Some(alt) = stats.values.get("alternative").and_then(|v| v.as_str()) {
        println!("{:<20} {:>15}", "Alternative", alt);
    }

    println!();
    if let (Some(ci_lo), Some(ci_hi), Some(conf)) = (
        stats.values.get("ci_lower").and_then(|v| v.as_f64()),
        stats.values.get("ci_upper").and_then(|v| v.as_f64()),
        stats
            .values
            .get("confidence_level")
            .and_then(|v| v.as_f64()),
    ) {
        println!("{:.0}% CI: [{:.6}, {:.6}]", conf * 100.0, ci_lo, ci_hi);
    }
}

fn render_chi_square(stats: &sf_core::StatsArtifact) {
    let test = stats
        .values
        .get("test")
        .and_then(|v| v.as_str())
        .unwrap_or("chi_squared");
    println!("Test: {}", test);

    if let Some(col1) = stats.values.get("column1").and_then(|v| v.as_str()) {
        print!("Columns: {} ", col1);
        if let Some(col2) = stats.values.get("column2").and_then(|v| v.as_str()) {
            print!("x {}", col2);
        }
        println!();
    }

    println!();
    println!("{:<20} {:>15}", "Statistic", "Value");
    println!("{}", "-".repeat(37));

    let fields = [
        ("n", "N"),
        ("k", "Categories"),
        ("rows", "Rows"),
        ("cols", "Cols"),
        ("chi2_statistic", "χ²-statistic"),
        ("df", "df"),
        ("p_value", "p-value"),
        ("effect_size", "Effect Size"),
    ];

    for (key, label) in &fields {
        if let Some(val) = stats.values.get(*key) {
            println!("{:<20} {:>15}", label, fmt_opt_val(Some(val)));
        }
    }

    if let Some(es_type) = stats
        .values
        .get("effect_size_type")
        .and_then(|v| v.as_str())
    {
        println!("{:<20} {:>15}", "Effect Size Type", es_type);
    }

    // Print contingency table if present
    if let Some(table) = stats
        .values
        .get("contingency_table")
        .and_then(|v| v.as_array())
    {
        let col_labels = stats
            .values
            .get("col_labels")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let row_labels = stats
            .values
            .get("row_labels")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        println!();
        println!("Contingency Table:");
        print!("{:<12}", "");
        for label in &col_labels {
            print!("{:>10}", label);
        }
        println!();
        for (i, row) in table.iter().enumerate() {
            if let Some(row_arr) = row.as_array() {
                let label = row_labels.get(i).map(|s| s.as_str()).unwrap_or("?");
                print!("{:<12}", label);
                for val in row_arr {
                    print!("{:>10}", val.as_u64().unwrap_or(0));
                }
                println!();
            }
        }
    }
}

fn render_variance_test(stats: &sf_core::StatsArtifact) {
    println!("Test: Variance chi-squared test");

    if let Some(col) = stats.values.get("column").and_then(|v| v.as_str()) {
        println!("Column: {}", col);
    }

    println!();
    println!("{:<20} {:>15}", "Statistic", "Value");
    println!("{}", "-".repeat(37));

    let fields = [
        ("n", "N"),
        ("variance", "Variance"),
        ("std_dev", "Std Dev"),
        ("sigma2_0", "σ²₀"),
        ("chi2_statistic", "χ²-statistic"),
        ("df", "df"),
        ("p_value", "p-value"),
    ];

    for (key, label) in &fields {
        if let Some(val) = stats.values.get(*key) {
            println!("{:<20} {:>15}", label, fmt_opt_val(Some(val)));
        }
    }

    if let Some(alt) = stats.values.get("alternative").and_then(|v| v.as_str()) {
        println!("{:<20} {:>15}", "Alternative", alt);
    }

    println!();
    if let (Some(ci_lo), Some(ci_hi), Some(conf)) = (
        stats.values.get("ci_lower").and_then(|v| v.as_f64()),
        stats.values.get("ci_upper").and_then(|v| v.as_f64()),
        stats
            .values
            .get("confidence_level")
            .and_then(|v| v.as_f64()),
    ) {
        println!(
            "{:.0}% CI for variance: [{:.6}, {:.6}]",
            conf * 100.0,
            ci_lo,
            ci_hi
        );
    }
}

fn render_generic(stats: &sf_core::StatsArtifact) {
    for (key, val) in &stats.values {
        println!("  {}: {}", key, val);
    }
}

fn fmt_opt_val(v: Option<&serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::Number(n)) => {
            if let Some(f) = n.as_f64() {
                format!("{:.4}", f)
            } else {
                n.to_string()
            }
        }
        Some(serde_json::Value::Null) | None => "N/A".to_string(),
        Some(v) => v.to_string(),
    }
}
