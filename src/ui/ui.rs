use crate::ui::app::App;
use tui::{
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    style::{Style, Color, Modifier},
    text::{Spans, Span},
    backend::Backend,
    Frame,
};
use rand::{Rng, thread_rng};

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let size = frame.size();

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(50), // Orderbook
            Constraint::Percentage(10), // Prices
            Constraint::Percentage(15), // User balances
            Constraint::Percentage(15), // Updates
            Constraint::Percentage(10),  // Command line
        ]
        .as_ref(),
    )
    .split(size);

    // Produce just the bar data values first
    let bar_values: Vec<u64> = app.positions.iter()
        .chain(std::iter::repeat(&0u64))  // Chain an infinite iterator of zeros to the end
        .take(size.width as usize)        // Only take as many values as size.width
        .cloned()                         // Clone each value from the iterator to get ownership
        .collect();                       // Collect values into a new Vec<u64>

    // Now, produce the labels
    let step = 0.1;
    let labels: Vec<String> = (0..size.width)
        .map(|i| format!("{:.2}", step * i as f32))
        .collect();

    // Combine the two to produce the sample data
    let sample_data: Vec<(&str, u64)> = labels.iter()
    .map(AsRef::as_ref)
    .zip(bar_values.iter().cloned())
    .collect();

    let barchart = BarChart::default()
    .block(Block::default().title("Orderbook").borders(Borders::ALL))
    .bar_width(3)
    .bar_gap(1)
    .bar_style(Style::default().fg(Color::Rgb(79,74,162)))
    .value_style(Style::default().add_modifier(Modifier::DIM))
    .label_style(Style::default().fg(Color::White))
    .data(&sample_data)
    .max(100);

    frame.render_widget(barchart, chunks[0]);

    // 2. Render user balances
    let balances_text = format!(
        "USD Balance: {}\n\
         OSMO Balance: {}", 690, 450 // app.usd_balance, app.btc_balance
    );
    let block = Block::default().borders(Borders::ALL).title("User Balances");
    let para = Paragraph::new(balances_text).block(block);
    frame.render_widget(para, chunks[2]);

    // 3. Render dynamic updates
    let update_text = app.updates.iter()
        .rev()
        .filter(|&message| !message.is_empty())
        .map(|message| {
            Spans::from(Span::styled(
                format!("{}", message),
                Style::default().fg(Color::Green)
            ))
        })
        .collect::<Vec<Spans>>();

    let block = Block::default().borders(Borders::ALL).title("Updates");
    let para = Paragraph::new(update_text).block(block);
    frame.render_widget(para, chunks[3]);


    // 4. Render command line
    let input_text = Spans::from(Span::styled(
        format!("{}", app.command_line),
        Style::default().fg(Color::Yellow)
    ));
    let block = Block::default().borders(Borders::ALL).title("Command Line");
    let para = Paragraph::new(input_text).block(block);
    frame.render_widget(para, chunks[4]);
}
