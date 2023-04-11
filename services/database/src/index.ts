import _sourceMap from "@yin/common/src/sourceMap";
import { z } from "zod";

import { validateEnvVars } from "@yin/common";

const env = validateEnvVars(z.object({}));
