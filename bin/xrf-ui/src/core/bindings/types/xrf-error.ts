// Auto-generated rust bindings. Do not edit it manually.

/** Error produced by XRF tools and libraries. */
export type XrfError =
  | ({
      Assertion: {
        message: string;
      };
    } & {
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Asset: {
        message: string;
      };
    } & {
      Assertion?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Convert: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Format: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Verify: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
    })
  | ({
      NotImplemented: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Read: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Unexpected: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      NotFound: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Invalid: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Parsing: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Encoding: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      NoTerminator: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      UnknownLanguage: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      Verify?: never;
    })
  | ({
      InvalidSource: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Serialization: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      TextureProcessing: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      ChunkNotEnded: {
        message: string;
        remaining: number;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      LtxParse: {
        line: number;
        col: number;
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      LtxScheme: {
        section: string;
        field: string;
        message: string;
        at: string | null;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  /**
   * An operation stopped at a safe boundary because it was asked to.
   *
   * Control flow rather than a failure: it exists so a cancellation check composes with `?` and can break a parallel
   * iterator, which stops on an error and on nothing else. An operation is expected to catch its own and report what
   * it completed, so this reaching a caller means one forgot to.
   */
  | ({
      Cancelled: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Generic: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Serde: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      Io?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    })
  | ({
      Io: {
        message: string;
      };
    } & {
      Assertion?: never;
      Asset?: never;
      Cancelled?: never;
      ChunkNotEnded?: never;
      Convert?: never;
      Encoding?: never;
      Format?: never;
      Generic?: never;
      Invalid?: never;
      InvalidSource?: never;
      LtxParse?: never;
      LtxScheme?: never;
      NoTerminator?: never;
      NotFound?: never;
      NotImplemented?: never;
      Parsing?: never;
      Read?: never;
      Serde?: never;
      Serialization?: never;
      TextureProcessing?: never;
      Unexpected?: never;
      UnknownLanguage?: never;
      Verify?: never;
    });
