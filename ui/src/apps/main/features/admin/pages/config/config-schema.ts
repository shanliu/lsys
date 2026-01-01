import { z } from 'zod';

export const ConfigParamSchema = z.object({
  type: z.string(),
});
export type ConfigParamType = z.infer<typeof ConfigParamSchema>;
