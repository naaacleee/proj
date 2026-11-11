package com.mentor.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.mentor.app.navigation.MentorNavHost
import com.mentor.core.ui.theme.MentorTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val app = application as MentorApplication
        setContent {
            MentorTheme {
                MentorNavHost(container = app.container)
            }
        }
    }
}
