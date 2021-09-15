import http, { AxiosResponse, Method } from "axios";


export default async function homeAssistantHttp(method: Method, url: string, data: Record<string, string | number>): Promise<AxiosResponse<unknown>> {
  return (await http({
    method,
    url: `http://${process.env.HA_URL}/api/${url}`,
    headers: {
      "authorization": `Bearer ${process.env.HA_TOKEN}`,
      "content-type": "application/json",
    },
    data,
  })).data;
}