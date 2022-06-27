import Snowflake from "../Snowflake";

export const defineSnowflake = (value: any): Snowflake | null => {
    return value ? new Snowflake(value) : null;
};
