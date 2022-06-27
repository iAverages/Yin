import Snowflake from "../Snowflake";

export const defineSnowflake = (value: any): Snowflake | undefined => {
    return value ? new Snowflake(value) : undefined;
};
