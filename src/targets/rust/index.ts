import fs from "fs"
import _ from "lodash"
import hbs from "handlebars"

import { typeResolvers } from "targets"
import {
  Target,
  TargetTypeMap,
  TargetServer,
  GenerateArguments
} from "types"
import {
  SchemaObject,
  ServerObject
} from "openapi3-ts"
import { Operation } from "visitor"

const apiTmpl = hbs.compile(fs.readFileSync(`${__dirname}/api.hbs`, "utf8"))
const reservedWords = fs.readFileSync(`${__dirname}/reserved-words.txt`, "utf8")
  .trim()
  .split("\n")

function genComment(indent: number, content: string): string {
  const pre = Array(indent + 1).join(" ")
  return `/// ${content.trim().split("\n").join(`\n${pre}/// `)}`
}

export default class RustTarget extends Target {
  types: TargetTypeMap = typeResolvers("rust")

  cls(key: string, isNested?: boolean | undefined): string {
    !key && console.trace("wat")
    const candidate = _.upperFirst(_.camelCase(key.replace("@", "at_")))

    if (reservedWords.includes(candidate)) {
      return `${candidate}_`
    }

    return candidate
  }

  enumKey(string: string): string {
    return this.cls(string)
  }

  oneOfKey(string: string): string {
    return this.cls(string)
  }

  modelDoc(schema: SchemaObject): string {
    if (schema.description == null) {
      return ""
    }

    return genComment(4, schema.description)
  }

  fieldDoc(schema: SchemaObject): string {
    if (schema.description == null) {
      return ""
    }

    return genComment(8, schema.description)
  }

  variable(basename: string): string {
    const candidate = _.snakeCase(basename.replace("@", "at_"))

    if (reservedWords.includes(candidate)) {
      return `${candidate}_`
    }

    return candidate
  }

  isHashable(type: string): boolean {
    return true
  }

  operationId(route: SchemaObject): string {
    return this.variable(route.operationId || route.summary)
  }

  pathUrl(routePath: string): string {
    return routePath
  }

  httpMethod(method: string): string {
    return method
  }

  url(thing: string): string {
    // TODO
    return thing
  }

  servers(servers: ServerObject[]): TargetServer[] {
    // TODO
    return []
  }

  generate(args: GenerateArguments): { [filename: string]: string; } {
    return { "generated.rs": apiTmpl(args), "generated.json": JSON.stringify(args, null, 2) }
  }

  operationParams(route: Operation, bodyName: string, paramNames: { [key: string]: string }): string {
    return "TODO"
  }
}
