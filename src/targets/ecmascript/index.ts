import fs from "fs"
import _ from "lodash"
import hbs from "handlebars"

import {
  Target,
  TargetTypeMap,
  TargetServer,
  GenerateArguments
} from "types"
import {
  SchemaObject,
  ParameterObject,
  ServerObject,
  ReferenceObject
} from "openapi3-ts"
import { Operation } from "visitor"

const apiTmpl = hbs.compile(fs.readFileSync(`${__dirname}/api.hbs`, "utf8"))

export default class EcmaScriptTarget extends Target {
  types: TargetTypeMap = new Proxy({}, {
    get(target: any, propertyKey: PropertyKey, receiver: any) {
      if (typeof propertyKey === "string" && ["null", "map", "set", "array"].includes(propertyKey)) {
        return ""
      }
      return new Proxy({}, {
        get(target: any, propertyKey: PropertyKey, receiver: any) {
          return ""
        }
      })
    }
  })

  cls(name: string): string {
    return _.upperFirst(_.camelCase(name))
  }

  enumKey(name: string): string {
    return this.cls(name)
  }

  variable(name: string): string {
    return _.camelCase(name)
  }

  optional(name: string): string {
    return name
  }

  fieldDoc(doc: SchemaObject): string {
    return "// " + doc
  }

  modelDoc(doc: SchemaObject): string {
    return "// " + doc
  }

  interface(name: string): string {
    return this.cls(name)
  }

  oneOfKey(name: string): string {
    return this.cls(name)
  }

  isHashable(type: string): boolean {
    return false
  }

  enum(name: string): string {
    return this.cls(name)
  }

  operationId(route: SchemaObject): string {
    if (route.operationId) {
      return this.variable(route.operationId)
    }

    return this.variable(route.summary)
  }

  httpMethod(name: string): string {
    return name.toUpperCase()
  }

  pathUrl(name: string): string {
    return name.substring(1).replace(/{/g, "${")
  }

  url(thing: string): string {
    const url = thing.replace(/{/g, "${")
    if (url.endsWith("/")) {
      return url
    }
    return `${url}/`
  }
  
  // security(items) {
  //   return items.map((x) => {
  //     const prop = Object.keys(x)[0]
  //     return {
  //       name: jsTarget.cls(prop),
  //       values: x[prop]
  //     }
  //   })
  // },
  
  requestParams(route: SchemaObject): string {
    let x: string[] = []

    if (route.parameters) {
      x = route.parameters.filter((p: ParameterObject) => {
        return p.in === "query"
      }).map((p: ParameterObject) => {
        const v = this.variable(p.name)
        return `if (${v} != null) __url.searchParams.set("${p.name}", ${v})`
      })
    }

    if (route.requestBody) {
      const requestBodySchema = route.requestBody as SchemaObject;
      const mimeType = route.requestMediaType as string;

      x.push(`__reqBody.headers = { "Content-Type": "${mimeType}" }`)

      if (mimeType.endsWith("form-data")) {
        
        if (!requestBodySchema.properties) {
          throw new Error(`Unexpected structure: Schema properties are mising`)
        }
        // TODO: this should be consistent across platforms
        
        const lines = Object.keys(requestBodySchema.properties).map((key) => {
          const v = this.variable(key)

          if (requestBodySchema.required && requestBodySchema.required.indexOf(key) > -1) {
            return `__formData.append("${key}", body.${v})`
          }

          return `if (${v} != null) {
            __formData.append("${key}", body.${v})
          }`
          
        }).join("\n")

        x.push(`
          const __formData = new FormData()
          ${lines}
          __reqBody.body = __formData
        `)
      } else {
        x.push("__reqBody.body = JSON.stringify(body)")
      }
    }

    return x.join("\n    ")
  }

  private isParameterObject(p: ParameterObject | ReferenceObject): p is ParameterObject {
    return typeof (p as any).$ref === "undefined"
  }

  operationParams(route: Operation, bodyName: string, paramNames: { [key: string]: string }) {
    let x: string[] = []
    
    if (route.parameters) {
      x = route.parameters
        .filter(this.isParameterObject)
        .map((p) => `${_.camelCase(p.name)}`)
    }

    if (route.requestBody) {
      // const k = Object.keys(route.requestBody.content)
      x.push(`body`)
    }

    if (x.length === 0) {
      return "()"
    }
    
    if (x.length === 1) {
      return `(${x[0]})`
    }

    return `({ ${x.join(", ")} })`
  }

  servers(servers: ServerObject[]): TargetServer[] {
    return servers.map((server, i) => ({
      url: this.url(server.url),
      description: this.variable(server.description || `default${i}`),
      variables: _.map(server.variables, (v, k) => {
        return `${this.variable(k)}`
      }).join(",\n        "),
      replacements: _.map(server.variables, (v, k) => {
        return {
          key: `{${k}}`,
          value: this.variable(k)
        }
      })
    }))
  }

  generate(args: GenerateArguments): { [filename: string]: string } {
    return { "Generated.js": apiTmpl(args) }
  }
}
