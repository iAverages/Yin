import * as mongoose from "mongoose";

export class Mongo {
    private mongnoConnection: mongoose.Connection;

    constructor() {
        mongoose.connect(process.env.MONGO_URL, {
            useNewUrlParser: true,
            useUnifiedTopology: true,
            useFindAndModify: false,
            useCreateIndex: true,
        });
        this.mongnoConnection = mongoose.connection;
    }

    public get connection() {
        return this.mongnoConnection;
    }
}
