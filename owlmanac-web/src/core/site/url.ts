/** Prefixes a site path with the base path: url("posts/x/") is "/owlmanac/posts/x/". */
export function url(path = ""): string {
  return import.meta.env.BASE_URL.replace(/\/?$/, "/") + path.replace(/^\//, "");
}
