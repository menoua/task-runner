import { desktopDir, homeDir, join } from "@tauri-apps/api/path";
import { createDir, readDir } from "@tauri-apps/api/fs";

export async function setupOutputDir(projectId, subjectId): Promise<string> {
    let exists: boolean
    let baseDir: string
    let outputDir: string
    let errorMessage: string = null

    await desktopDir().then(
        value => baseDir = value,
        error => homeDir().then(
            value => baseDir = value,
            error => {
                alert(error)
                errorMessage = 'Could not find a Desktop or Home directory.'
            }
        ),
    )

    if (errorMessage !== null) { return Promise.reject(errorMessage) }

    outputDir = await join(baseDir, 'TaskRunner.out')
    await readDir(outputDir).then(
        value => exists = true,
        error => exists = false,
    )

    if (!exists) {
        await createDir(outputDir).catch(
            error => {
                alert(error)
                errorMessage = 'Failed to create output directory: ' + outputDir
            }
        )
    }

    if (errorMessage !== null) { return Promise.reject(errorMessage) }

    outputDir = await join(outputDir, projectId)
    await readDir(outputDir).then(
        value => exists = true,
        error => exists = false,
    )

    if (!exists) {
        await createDir(outputDir).catch(
            error => {
                alert(error)
                errorMessage = 'Failed to create project directory: ' + outputDir
            }
        )
    }

    if (errorMessage !== null) { return Promise.reject(errorMessage) }

    outputDir = await join(outputDir, subjectId)
    await readDir(outputDir).then(
        value => exists = true,
        error => exists = false,
    )

    if (!exists) {
        await createDir(outputDir).catch(
            error => {
                alert(error)
                errorMessage = 'Failed to create subject directory: ' + outputDir
            }
        )
    }

    if (errorMessage !== null) { return Promise.reject(errorMessage) }

    return Promise.resolve(outputDir)
}

export async function setupBlockDir(outputDir, blockId): Promise<string> {
    let exists: boolean
    let activeDir: string
    let errorMessage: string = null

    return Promise.reject('Not implemented')
}
