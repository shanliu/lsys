import { z } from 'zod';

export const AdapterConfigParamSchema = z.object({
  type: z.string(),
});
export type AdapterConfigParamType = z.infer<typeof AdapterConfigParamSchema>;
