use dynamic_tournament_test::TournamentGenerator;

use dynamic_tournament_api::Client;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);

    let host = args.next().unwrap();
    let username = args.next().unwrap();
    let password = args.next().unwrap();
    let entrants: usize = args.next().unwrap().parse().unwrap();

    let client = Client::new(host);
    client
        .v3()
        .auth()
        .login(&username, &password)
        .await
        .unwrap();

    let mut generator = TournamentGenerator::new();
    generator.entrants = entrants;

    let tournament = generator.generate();

    client.tournaments().create(&tournament).await.unwrap();
}
