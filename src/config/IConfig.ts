export interface IConfig {
    database: {
        driver: string;
        mongo: {
            url: string;
        };
        mysql: {
            host: string;
            port: number;
            user: string;
            password: string;
            connectionLimit: number;
            database: string;
        };
    };
    guild: {
        defaults: {
            prefix: string;
        };
    };
}
