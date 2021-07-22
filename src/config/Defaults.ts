enum Defaults {
    // Database settings
    "database.driver" = "mongo",

    // Mongo driver settings
    "database.mongo.uri" = "mongodb://localhost:27017/discord_bot",

    // MySQL driver settings
    "database.mysql.host" = "localhost",
    "database.mysql.port" = 3306,
    "database.mysql.user" = "root",
    "database.mysql.password" = "password",
    "database.mysql.connectionLimit" = 5,
    "database.mysql.database" = "discord_bot",

    // General settings
    "guild.defaults.prefix" = "-",
}

export default Defaults;
